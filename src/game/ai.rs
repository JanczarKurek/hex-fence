use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use crate::app_state::{AiStrategy, GameConfig};
use crate::hex_grid::AxialCoord;
use crate::network::{NetConfig, NetRuntime};

use super::actions::{ActionSource, GameActionRequest};
use super::fence::{FenceShape, fence_edges};
use super::state::{ActionOutcome, EdgeKey, GameAction, TurnState};

#[derive(Resource)]
pub struct AiRng {
    state: u64,
}

impl Default for AiRng {
    fn default() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);
        Self { state: seed | 1 }
    }
}

impl AiRng {
    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state = self.state.wrapping_mul(0x2545F4914F6CDD1D);
        self.state
    }

    fn choose_index(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len
    }
}

#[derive(Resource)]
pub struct AiTurnCooldown {
    pending_player: Option<usize>,
    timer: Timer,
}

impl Default for AiTurnCooldown {
    fn default() -> Self {
        Self {
            pending_player: None,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

impl AiTurnCooldown {
    fn clear(&mut self) {
        self.pending_player = None;
        self.timer.reset();
    }

    fn start_for_player(&mut self, player: usize, cooldown_seconds: f32) {
        self.pending_player = Some(player);
        self.timer
            .set_duration(Duration::from_secs_f32(cooldown_seconds.max(0.0)));
        self.timer.reset();
    }
}

pub fn random_ai_take_turn(
    time: Res<Time>,
    game_config: Res<GameConfig>,
    turn_state: Res<TurnState>,
    net_config: Res<NetConfig>,
    net_runtime: Res<NetRuntime>,
    mut ai_rng: ResMut<AiRng>,
    mut ai_cooldown: ResMut<AiTurnCooldown>,
    mut action_requests: EventWriter<GameActionRequest>,
) {
    if turn_state.winner.is_some() {
        ai_cooldown.clear();
        return;
    }

    let current_player = turn_state.current_player;
    if !game_config.player_control(current_player).is_ai() {
        ai_cooldown.clear();
        return;
    }

    if !net_runtime.can_control_player(&net_config, current_player) {
        ai_cooldown.clear();
        return;
    }

    if ai_cooldown.pending_player != Some(current_player) {
        ai_cooldown.start_for_player(current_player, game_config.ai_cooldown_seconds);
        return;
    }

    ai_cooldown.timer.tick(time.delta());
    if !ai_cooldown.timer.finished() {
        return;
    }

    let action = match game_config.ai_strategy {
        AiStrategy::Heuristic => choose_heuristic_action(&turn_state, &mut ai_rng),
        AiStrategy::AlphaBeta => choose_alpha_beta_action(&turn_state, &mut ai_rng, 3)
            .or_else(|| choose_heuristic_action(&turn_state, &mut ai_rng)),
    };

    let Some(action) = action else {
        ai_cooldown.clear();
        return;
    };

    ai_cooldown.clear();
    action_requests.write(GameActionRequest {
        source: ActionSource::Local,
        action,
    });
}

fn choose_heuristic_action(turn_state: &TurnState, ai_rng: &mut AiRng) -> Option<GameAction> {
    let current = turn_state.current_player;
    let goal_side = turn_state.players[current].goal_side;
    let base_self_path = shortest_path_len(
        turn_state.pawn_positions[current],
        goal_side,
        turn_state.board_radius,
        &turn_state.blocked_edges,
    )? as i32;

    let legal_moves = turn_state.legal_moves_for_current();
    if legal_moves.is_empty() {
        return None;
    }

    let mut winning_moves = Vec::new();
    let mut best_move_score = i32::MIN;
    let mut best_move = legal_moves[ai_rng.choose_index(legal_moves.len())];

    for target in legal_moves {
        if target.is_on_side(goal_side, turn_state.board_radius) {
            winning_moves.push(target);
            continue;
        }

        let move_path = shortest_path_len(
            target,
            goal_side,
            turn_state.board_radius,
            &turn_state.blocked_edges,
        )
        .unwrap_or(i32::MAX as u32) as i32;
        let progress = base_self_path - move_path;
        let score = progress * 120 - move_path * 10;

        if score > best_move_score || (score == best_move_score && ai_rng.next_u64() & 1 == 0) {
            best_move_score = score;
            best_move = target;
        }
    }

    if !winning_moves.is_empty() {
        let win_target = winning_moves[ai_rng.choose_index(winning_moves.len())];
        return Some(GameAction::Move { target: win_target });
    }

    let opponent_paths = opponent_paths(turn_state, &turn_state.blocked_edges);
    let closest_opponent = opponent_paths
        .iter()
        .map(|(_, path)| *path)
        .min()
        .unwrap_or(i32::MAX);

    if turn_state.fences_left[current] > 0
        && (closest_opponent <= base_self_path || closest_opponent <= 2)
        && let Some(edges) = choose_best_fence(turn_state, base_self_path, &opponent_paths, ai_rng)
    {
        return Some(GameAction::PlaceFence { edges });
    }

    Some(GameAction::Move { target: best_move })
}

fn choose_alpha_beta_action(
    turn_state: &TurnState,
    ai_rng: &mut AiRng,
    depth: usize,
) -> Option<GameAction> {
    let root_player = turn_state.current_player;
    let actions = ordered_candidate_actions(turn_state, ai_rng);
    if actions.is_empty() {
        return None;
    }

    let mut best_action = actions[ai_rng.choose_index(actions.len())];
    let mut best_score = i32::MIN;
    let mut alpha = i32::MIN / 2;
    let beta = i32::MAX / 2;

    for action in actions {
        let mut simulated = turn_state.clone();
        let Ok(outcome) = simulated.try_apply_action(action) else {
            continue;
        };

        let score = match outcome {
            super::state::AppliedAction::Moved {
                outcome: ActionOutcome::Won(_),
                ..
            } => 1_000_000,
            _ => alpha_beta(
                &simulated,
                root_player,
                depth.saturating_sub(1),
                alpha,
                beta,
                ai_rng,
            ),
        };

        if score > best_score || (score == best_score && ai_rng.next_u64() & 1 == 0) {
            best_score = score;
            best_action = action;
        }
        alpha = alpha.max(score);
    }

    Some(best_action)
}

fn alpha_beta(
    turn_state: &TurnState,
    root_player: usize,
    depth: usize,
    mut alpha: i32,
    mut beta: i32,
    ai_rng: &mut AiRng,
) -> i32 {
    if depth == 0 || turn_state.winner.is_some() {
        return evaluate_position(turn_state, root_player);
    }

    let actions = ordered_candidate_actions(turn_state, ai_rng);
    if actions.is_empty() {
        return evaluate_position(turn_state, root_player);
    }

    if turn_state.current_player == root_player {
        let mut value = i32::MIN;
        for action in actions {
            let mut simulated = turn_state.clone();
            if simulated.try_apply_action(action).is_err() {
                continue;
            }
            let score = alpha_beta(&simulated, root_player, depth - 1, alpha, beta, ai_rng);
            value = value.max(score);
            alpha = alpha.max(value);
            if beta <= alpha {
                break;
            }
        }
        value
    } else {
        let mut value = i32::MAX;
        for action in actions {
            let mut simulated = turn_state.clone();
            if simulated.try_apply_action(action).is_err() {
                continue;
            }
            let score = alpha_beta(&simulated, root_player, depth - 1, alpha, beta, ai_rng);
            value = value.min(score);
            beta = beta.min(value);
            if beta <= alpha {
                break;
            }
        }
        value
    }
}

fn evaluate_position(turn_state: &TurnState, root_player: usize) -> i32 {
    if let Some(winner) = turn_state.winner {
        return if winner == root_player {
            1_000_000
        } else {
            -1_000_000
        };
    }

    let root_path = shortest_path_len(
        turn_state.pawn_positions[root_player],
        turn_state.players[root_player].goal_side,
        turn_state.board_radius,
        &turn_state.blocked_edges,
    )
    .unwrap_or(u32::MAX / 8) as i32;

    let mut best_opponent_path = i32::MAX / 8;
    let mut opponent_path_sum = 0;
    let mut opponent_count = 0;
    for player in 0..turn_state.players.len() {
        if player == root_player {
            continue;
        }
        let path = shortest_path_len(
            turn_state.pawn_positions[player],
            turn_state.players[player].goal_side,
            turn_state.board_radius,
            &turn_state.blocked_edges,
        )
        .unwrap_or(u32::MAX / 8) as i32;
        best_opponent_path = best_opponent_path.min(path);
        opponent_path_sum += path;
        opponent_count += 1;
    }

    let opponent_avg = if opponent_count == 0 {
        best_opponent_path
    } else {
        opponent_path_sum / opponent_count
    };

    let root_fences = turn_state.fences_left[root_player] as i32;
    let opponent_fences = turn_state
        .fences_left
        .iter()
        .enumerate()
        .filter(|(player, _)| *player != root_player)
        .map(|(_, fences)| *fences as i32)
        .sum::<i32>();

    (best_opponent_path - root_path) * 140
        + (opponent_avg - root_path) * 40
        + (root_fences - opponent_fences / opponent_count.max(1) as i32) * 4
}

fn ordered_candidate_actions(turn_state: &TurnState, ai_rng: &mut AiRng) -> Vec<GameAction> {
    let mut scored_moves = Vec::new();
    let current = turn_state.current_player;
    let goal_side = turn_state.players[current].goal_side;
    let base_self_path = shortest_path_len(
        turn_state.pawn_positions[current],
        goal_side,
        turn_state.board_radius,
        &turn_state.blocked_edges,
    )
    .unwrap_or(i32::MAX as u32) as i32;

    for target in turn_state.legal_moves_for_current() {
        if target.is_on_side(goal_side, turn_state.board_radius) {
            scored_moves.push((10_000, GameAction::Move { target }));
            continue;
        }
        let path_after = shortest_path_len(
            target,
            goal_side,
            turn_state.board_radius,
            &turn_state.blocked_edges,
        )
        .unwrap_or(i32::MAX as u32) as i32;
        let score = (base_self_path - path_after) * 120 - path_after * 10;
        scored_moves.push((score, GameAction::Move { target }));
    }

    scored_moves.sort_unstable_by_key(|(score, _)| -*score);
    let mut actions: Vec<GameAction> = scored_moves.into_iter().map(|(_, action)| action).collect();

    if let Some(fence_action) = best_fence_action(turn_state, ai_rng) {
        actions.insert(0, fence_action);
    }

    actions
}

fn best_fence_action(turn_state: &TurnState, ai_rng: &mut AiRng) -> Option<GameAction> {
    let current = turn_state.current_player;
    if turn_state.fences_left[current] == 0 {
        return None;
    }

    let base_self_path = shortest_path_len(
        turn_state.pawn_positions[current],
        turn_state.players[current].goal_side,
        turn_state.board_radius,
        &turn_state.blocked_edges,
    )? as i32;
    let opponent_paths = opponent_paths(turn_state, &turn_state.blocked_edges);
    choose_best_fence(turn_state, base_self_path, &opponent_paths, ai_rng)
        .map(|edges| GameAction::PlaceFence { edges })
}

fn choose_best_fence(
    turn_state: &TurnState,
    base_self_path: i32,
    opponent_paths_before: &[(usize, i32)],
    ai_rng: &mut AiRng,
) -> Option<[EdgeKey; 3]> {
    let current = turn_state.current_player;
    let mut best_edges: Option<[EdgeKey; 3]> = None;
    let mut best_score = i32::MIN;

    for edges in candidate_fences(turn_state, opponent_paths_before) {
        if !turn_state.can_place_fence(&edges) {
            continue;
        }

        let mut future_blocked = turn_state.blocked_edges.clone();
        for edge in edges {
            future_blocked.insert(edge);
        }

        let self_after = shortest_path_len(
            turn_state.pawn_positions[current],
            turn_state.players[current].goal_side,
            turn_state.board_radius,
            &future_blocked,
        )
        .unwrap_or(i32::MAX as u32) as i32;
        let self_delta = self_after - base_self_path;

        let mut best_opp_delta = 0;
        let mut total_opp_delta = 0;
        for (opponent, before) in opponent_paths_before.iter().copied() {
            let after = shortest_path_len(
                turn_state.pawn_positions[opponent],
                turn_state.players[opponent].goal_side,
                turn_state.board_radius,
                &future_blocked,
            )
            .unwrap_or(i32::MAX as u32) as i32;
            let delta = after - before;
            best_opp_delta = best_opp_delta.max(delta);
            total_opp_delta += delta;
        }

        let score = best_opp_delta * 100 + total_opp_delta * 30 - self_delta * 130;
        if score > best_score || (score == best_score && ai_rng.next_u64() & 1 == 0) {
            best_score = score;
            best_edges = Some(edges);
        }
    }

    if best_score > 0 { best_edges } else { None }
}

fn candidate_fences(
    turn_state: &TurnState,
    opponent_paths_before: &[(usize, i32)],
) -> Vec<[EdgeKey; 3]> {
    let mut anchors = Vec::new();
    for (opponent, _) in opponent_paths_before.iter().copied() {
        anchors.extend(hex_area(
            turn_state.pawn_positions[opponent],
            2,
            turn_state.board_radius,
        ));
    }
    anchors.extend(hex_area(
        turn_state.pawn_positions[turn_state.current_player],
        1,
        turn_state.board_radius,
    ));

    let mut dedup = HashSet::new();
    for anchor in anchors {
        for shape in [FenceShape::C, FenceShape::Y, FenceShape::S] {
            for orientation in 0..6 {
                dedup.insert(canonical_edges(fence_edges(anchor, shape, orientation)));
            }
        }
    }

    dedup.into_iter().collect()
}

fn canonical_edges(mut edges: [EdgeKey; 3]) -> [EdgeKey; 3] {
    edges.sort_unstable_by_key(|edge| (edge.a.q, edge.a.r, edge.b.q, edge.b.r));
    edges
}

fn hex_area(center: AxialCoord, distance: i32, board_radius: i32) -> Vec<AxialCoord> {
    let mut result = Vec::new();
    for q in (center.q - distance)..=(center.q + distance) {
        for r in (center.r - distance)..=(center.r + distance) {
            let coord = AxialCoord::new(q, r);
            if coord.is_inside_board(board_radius) && hex_distance(center, coord) <= distance {
                result.push(coord);
            }
        }
    }
    result
}

fn hex_distance(a: AxialCoord, b: AxialCoord) -> i32 {
    let dq = (a.q - b.q).abs();
    let dr = (a.r - b.r).abs();
    let ds = ((-a.q - a.r) - (-b.q - b.r)).abs();
    (dq + dr + ds) / 2
}

fn opponent_paths(turn_state: &TurnState, blocked_edges: &HashSet<EdgeKey>) -> Vec<(usize, i32)> {
    let current = turn_state.current_player;
    let mut result = Vec::new();
    for player in 0..turn_state.players.len() {
        if player == current {
            continue;
        }
        let path = shortest_path_len(
            turn_state.pawn_positions[player],
            turn_state.players[player].goal_side,
            turn_state.board_radius,
            blocked_edges,
        )
        .unwrap_or(i32::MAX as u32) as i32;
        result.push((player, path));
    }
    result
}

fn shortest_path_len(
    start: AxialCoord,
    goal_side: usize,
    board_radius: i32,
    blocked_edges: &HashSet<EdgeKey>,
) -> Option<u32> {
    if start.is_on_side(goal_side, board_radius) {
        return Some(0);
    }

    let mut visited = HashSet::from([start]);
    let mut queue = VecDeque::from([(start, 0u32)]);

    while let Some((current, distance)) = queue.pop_front() {
        for neighbor in current.neighbors() {
            if !neighbor.is_inside_board(board_radius) {
                continue;
            }
            if blocked_edges.contains(&EdgeKey::from_cells(current, neighbor)) {
                continue;
            }
            if !visited.insert(neighbor) {
                continue;
            }
            if neighbor.is_on_side(goal_side, board_radius) {
                return Some(distance + 1);
            }
            queue.push_back((neighbor, distance + 1));
        }
    }

    None
}
