use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use crate::app_state::GameConfig;
use crate::hex_grid::AxialCoord;

use super::player::{PlayerDef, fences_per_player, players_for_count};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeKey {
    pub a: AxialCoord,
    pub b: AxialCoord,
}

impl EdgeKey {
    pub fn from_cells(a: AxialCoord, b: AxialCoord) -> Self {
        if (a.q, a.r) <= (b.q, b.r) {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

#[derive(Resource, Clone)]
pub struct TurnState {
    pub board_radius: i32,
    pub players: Vec<PlayerDef>,
    pub pawn_positions: Vec<AxialCoord>,
    pub current_player: usize,
    pub winner: Option<usize>,
    pub fences_left: Vec<usize>,
    pub blocked_edges: HashSet<EdgeKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    GameFinished,
    IllegalMove,
    IllegalFencePlacement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionOutcome {
    TurnAdvanced,
    Won(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameAction {
    Move { target: AxialCoord },
    PlaceFence { edges: [EdgeKey; 3] },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppliedAction {
    Moved {
        player: usize,
        target: AxialCoord,
        outcome: ActionOutcome,
    },
    FencePlaced {
        player: usize,
        edges: [EdgeKey; 3],
    },
}

impl Default for TurnState {
    fn default() -> Self {
        Self::new(3, 4)
    }
}

impl TurnState {
    pub fn new(player_count: usize, board_radius: i32) -> Self {
        let default_config = GameConfig::default();
        Self::with_player_colors(player_count, board_radius, &default_config.player_colors)
    }

    pub fn with_player_colors(
        player_count: usize,
        board_radius: i32,
        player_colors: &[crate::app_state::PlayerColor; 6],
    ) -> Self {
        let players = players_for_count(player_count, player_colors);
        let pawn_positions = players
            .iter()
            .map(|player| player.start_coord(board_radius))
            .collect::<Vec<_>>();
        let fences_left = vec![fences_per_player(players.len()); players.len()];

        Self {
            board_radius,
            players,
            pawn_positions,
            current_player: 0,
            winner: None,
            fences_left,
            blocked_edges: HashSet::new(),
        }
    }

    pub fn is_occupied(&self, coord: AxialCoord) -> bool {
        self.pawn_positions.iter().any(|current| *current == coord)
    }

    pub fn advance_turn(&mut self) {
        self.current_player = (self.current_player + 1) % self.players.len();
    }

    fn can_step(&self, from: AxialCoord, to: AxialCoord) -> bool {
        if !to.is_inside_board(self.board_radius) {
            return false;
        }

        if from.direction_to(to).is_none() {
            return false;
        }

        !self.blocked_edges.contains(&EdgeKey::from_cells(from, to))
    }

    pub fn legal_moves_for_current(&self) -> Vec<AxialCoord> {
        let current_pos = self.pawn_positions[self.current_player];
        let mut legal_moves = HashSet::new();

        for neighbor in current_pos.neighbors() {
            if !self.can_step(current_pos, neighbor) {
                continue;
            }

            if !self.is_occupied(neighbor) {
                legal_moves.insert(neighbor);
                continue;
            }

            let mut queue = VecDeque::from([(current_pos, neighbor)]);
            let mut visited = HashSet::from([(current_pos, neighbor)]);

            while let Some((from, occupied)) = queue.pop_front() {
                let direction = from
                    .direction_to(occupied)
                    .expect("occupied pawn must be adjacent to source");
                let straight = occupied.neighbor_in_direction(direction);

                let candidates: Vec<AxialCoord> = if self.can_step(occupied, straight) {
                    vec![straight]
                } else {
                    occupied
                        .neighbors()
                        .into_iter()
                        .filter(|candidate| {
                            *candidate != from && self.can_step(occupied, *candidate)
                        })
                        .collect()
                };

                for candidate in candidates {
                    if self.is_occupied(candidate) {
                        let pair = (occupied, candidate);
                        if visited.insert(pair) {
                            queue.push_back(pair);
                        }
                    } else {
                        legal_moves.insert(candidate);
                    }
                }
            }
        }

        legal_moves.into_iter().collect()
    }

    pub fn can_place_fence(&self, edges: &[EdgeKey; 3]) -> bool {
        if self.fences_left[self.current_player] == 0 {
            return false;
        }

        let unique_edges: HashSet<EdgeKey> = edges.iter().copied().collect();
        if unique_edges.len() != 3 {
            return false;
        }

        for edge in edges {
            if !edge.a.is_inside_board(self.board_radius)
                || !edge.b.is_inside_board(self.board_radius)
            {
                return false;
            }

            if edge.a.direction_to(edge.b).is_none() {
                return false;
            }

            if self.blocked_edges.contains(edge) {
                return false;
            }
        }

        let mut future_blocked = self.blocked_edges.clone();
        for edge in edges {
            future_blocked.insert(*edge);
        }

        for (index, player) in self.players.iter().enumerate() {
            if !has_path_to_goal(
                self.pawn_positions[index],
                player.goal_side,
                self.board_radius,
                &future_blocked,
            ) {
                return false;
            }
        }

        true
    }

    pub fn place_fence(&mut self, edges: &[EdgeKey; 3]) {
        for edge in edges {
            self.blocked_edges.insert(*edge);
        }
        self.fences_left[self.current_player] -= 1;
        self.advance_turn();
    }

    pub fn try_place_fence(&mut self, edges: &[EdgeKey; 3]) -> Result<(), ActionError> {
        if self.winner.is_some() {
            return Err(ActionError::GameFinished);
        }

        if !self.can_place_fence(edges) {
            return Err(ActionError::IllegalFencePlacement);
        }

        self.place_fence(edges);
        Ok(())
    }

    pub fn try_move_current_pawn(
        &mut self,
        target: AxialCoord,
    ) -> Result<ActionOutcome, ActionError> {
        if self.winner.is_some() {
            return Err(ActionError::GameFinished);
        }

        if !target.is_inside_board(self.board_radius) {
            return Err(ActionError::IllegalMove);
        }

        let current = self.current_player;
        let current_pos = self.pawn_positions[current];
        if target == current_pos {
            return Err(ActionError::IllegalMove);
        }

        let legal_moves = self.legal_moves_for_current();
        if !legal_moves.contains(&target) {
            return Err(ActionError::IllegalMove);
        }

        self.pawn_positions[current] = target;
        if target.is_on_side(self.players[current].goal_side, self.board_radius) {
            self.winner = Some(current);
            return Ok(ActionOutcome::Won(current));
        }

        self.advance_turn();
        Ok(ActionOutcome::TurnAdvanced)
    }

    pub fn try_apply_action(&mut self, action: GameAction) -> Result<AppliedAction, ActionError> {
        match action {
            GameAction::Move { target } => {
                let player = self.current_player;
                let outcome = self.try_move_current_pawn(target)?;
                Ok(AppliedAction::Moved {
                    player,
                    target,
                    outcome,
                })
            }
            GameAction::PlaceFence { edges } => {
                let player = self.current_player;
                self.try_place_fence(&edges)?;
                Ok(AppliedAction::FencePlaced { player, edges })
            }
        }
    }
}

pub fn reset_turn_state_from_config(
    game_config: Res<GameConfig>,
    mut turn_state: ResMut<TurnState>,
) {
    *turn_state = TurnState::with_player_colors(
        game_config.player_count,
        game_config.board_radius,
        &game_config.player_colors,
    );
}

fn has_path_to_goal(
    start: AxialCoord,
    goal_side: usize,
    board_radius: i32,
    blocked_edges: &HashSet<EdgeKey>,
) -> bool {
    let mut visited = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);

    while let Some(current) = queue.pop_front() {
        if current.is_on_side(goal_side, board_radius) {
            return true;
        }

        for neighbor in current.neighbors() {
            if !neighbor.is_inside_board(board_radius) {
                continue;
            }

            let edge = EdgeKey::from_cells(current, neighbor);
            if blocked_edges.contains(&edge) || !visited.insert(neighbor) {
                continue;
            }

            queue.push_back(neighbor);
        }
    }

    false
}
