//! PUCT Monte-Carlo Tree Search over [`TurnState`], guided by an [`Evaluator`].
//!
//! The search is generic over the position evaluator so the same tree code serves both
//! pure-Rust testing (via [`UniformEvaluator`]) and ONNX-backed neural inference. Values are
//! always tracked from the perspective of the player to move at each node; backup flips sign
//! based on whose turn it is — crucially **not** on ply parity, because a winning move does
//! not advance the turn in this engine (see [`backup`]).

use std::collections::HashSet;

use crate::encoding::Encoder;
use crate::heuristic::AiRng;
use crate::state::{EdgeKey, GameAction, TurnState};

/// Evaluates a canonical position, returning `(policy_logits, value)` where the policy is
/// over the full `policy_len` action space (raw logits — masking/softmax happen here) and
/// the value is in `[-1, 1]` from the side-to-move's perspective.
pub trait Evaluator {
    fn evaluate(&self, planes: &[f32]) -> (Vec<f32>, f32);
}

/// A trivial evaluator: uniform priors over legal actions, neutral value. Used to test the
/// tree mechanics without a neural network.
pub struct UniformEvaluator {
    pub policy_len: usize,
}

impl Evaluator for UniformEvaluator {
    fn evaluate(&self, _planes: &[f32]) -> (Vec<f32>, f32) {
        (vec![0.0; self.policy_len], 0.0)
    }
}

#[derive(Clone, Copy)]
pub struct MctsConfig {
    pub simulations: usize,
    pub c_puct: f32,
    /// Softmax temperature for the final move choice. `<= 0` means deterministic argmax.
    pub temperature: f32,
    /// Dirichlet noise concentration mixed into the root priors (self-play exploration).
    pub dirichlet_alpha: f32,
    /// Fraction of root prior taken from Dirichlet noise. `0` disables it (use for eval).
    pub dirichlet_epsilon: f32,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            simulations: 128,
            c_puct: 1.5,
            temperature: 0.0,
            dirichlet_alpha: 0.3,
            dirichlet_epsilon: 0.0,
        }
    }
}

pub struct MctsResult {
    /// Chosen action, in the original (un-rotated) frame, ready to apply to the real state.
    pub action: GameAction,
    /// Visit-count distribution over the canonical policy index space (the training target).
    pub policy_target: Vec<f32>,
    /// Root value estimate (expected outcome for the side to move), in `[-1, 1]`.
    pub root_value: f32,
}

struct Edge {
    /// Canonical policy-head index for this action.
    index: usize,
    action: GameAction,
    prior: f32,
    visits: u32,
    value_sum: f64,
    child: Option<usize>,
}

struct Node {
    to_move: usize,
    state: TurnState,
    edges: Vec<Edge>,
    expanded: bool,
    /// `Some(value)` from this node's side-to-move perspective if the game is over here.
    terminal: Option<f32>,
}

struct Tree<'a, E: Evaluator> {
    nodes: Vec<Node>,
    encoder: &'a Encoder,
    eval: &'a E,
    c_puct: f32,
}

impl<'a, E: Evaluator> Tree<'a, E> {
    fn new(encoder: &'a Encoder, eval: &'a E, c_puct: f32) -> Self {
        Self {
            nodes: Vec::new(),
            encoder,
            eval,
            c_puct,
        }
    }

    fn push_node(&mut self, state: TurnState) -> usize {
        let to_move = state.current_player;
        let terminal = state
            .winner
            .map(|winner| if winner == to_move { 1.0 } else { -1.0 });
        let id = self.nodes.len();
        self.nodes.push(Node {
            to_move,
            state,
            edges: Vec::new(),
            expanded: false,
            terminal,
        });
        id
    }

    /// Expand a leaf: evaluate it, build legal edges with softmax priors. Returns the node's
    /// value (side-to-move perspective).
    fn expand(&mut self, id: usize) -> f32 {
        let encoder = self.encoder;
        let eval = self.eval;

        let (planes, k) = encoder.encode(&self.nodes[id].state);
        let (logits, value) = eval.evaluate(&planes);
        let legal = encoder.enumerate_legal_fast(&self.nodes[id].state, k);

        // Dedup equivalent fences (same edge set, different anchor/orientation tuple) so we
        // never create two children for the same resulting position.
        let mut seen_fences: HashSet<[EdgeKey; 3]> = HashSet::new();
        let mut edges: Vec<Edge> = Vec::with_capacity(legal.len());
        for (index, action) in legal {
            if let GameAction::PlaceFence { edges: e } = action {
                let mut key = e;
                key.sort_unstable_by_key(|ek| (ek.a.q, ek.a.r, ek.b.q, ek.b.r));
                if !seen_fences.insert(key) {
                    continue;
                }
            }
            edges.push(Edge {
                index,
                action,
                prior: 0.0,
                visits: 0,
                value_sum: 0.0,
                child: None,
            });
        }

        softmax_priors(&mut edges, &logits);

        self.nodes[id].edges = edges;
        self.nodes[id].expanded = true;
        value
    }

    fn select_edge(&self, id: usize) -> usize {
        let node = &self.nodes[id];
        let total_visits: u32 = node.edges.iter().map(|e| e.visits).sum();
        let exploration = (1.0 + total_visits as f32).sqrt();

        let mut best = 0;
        let mut best_score = f32::MIN;
        for (i, edge) in node.edges.iter().enumerate() {
            let q = if edge.visits == 0 {
                0.0
            } else {
                (edge.value_sum / edge.visits as f64) as f32
            };
            let u = self.c_puct * edge.prior * exploration / (1.0 + edge.visits as f32);
            let score = q + u;
            if score > best_score {
                best_score = score;
                best = i;
            }
        }
        best
    }

    fn simulate(&mut self, root: usize) {
        let mut path: Vec<(usize, usize)> = Vec::new();
        let mut node = root;

        let (leaf_value, leaf_to_move) = loop {
            if let Some(terminal) = self.nodes[node].terminal {
                break (terminal, self.nodes[node].to_move);
            }
            if !self.nodes[node].expanded {
                let value = self.expand(node);
                break (value, self.nodes[node].to_move);
            }

            let edge = self.select_edge(node);
            path.push((node, edge));

            match self.nodes[node].edges[edge].child {
                Some(child) => node = child,
                None => {
                    let mut state = self.nodes[node].state.clone();
                    let action = self.nodes[node].edges[edge].action;
                    state
                        .try_apply_action(action)
                        .expect("tree edge must be a legal action");
                    let child = self.push_node(state);
                    self.nodes[node].edges[edge].child = Some(child);

                    let value = match self.nodes[child].terminal {
                        Some(terminal) => terminal,
                        None => self.expand(child),
                    };
                    break (value, self.nodes[child].to_move);
                }
            }
        };

        backup(&mut self.nodes, &path, leaf_value, leaf_to_move);
    }
}

/// Back up `leaf_value` (from `leaf_to_move`'s perspective) along the visited path. Each edge
/// records the value from *its parent's* side-to-move perspective: same sign when the parent
/// shares the leaf's side to move, flipped otherwise. This handles the winning-move case where
/// the turn does not alternate.
fn backup(nodes: &mut [Node], path: &[(usize, usize)], leaf_value: f32, leaf_to_move: usize) {
    for &(node, edge) in path {
        let value = if nodes[node].to_move == leaf_to_move {
            leaf_value
        } else {
            -leaf_value
        };
        let edge = &mut nodes[node].edges[edge];
        edge.visits += 1;
        edge.value_sum += value as f64;
    }
}

fn softmax_priors(edges: &mut [Edge], logits: &[f32]) {
    if edges.is_empty() {
        return;
    }
    let max = edges
        .iter()
        .map(|e| logits[e.index])
        .fold(f32::MIN, f32::max);
    let mut sum = 0.0f32;
    for edge in edges.iter_mut() {
        let p = (logits[edge.index] - max).exp();
        edge.prior = p;
        sum += p;
    }
    if sum > 0.0 {
        for edge in edges.iter_mut() {
            edge.prior /= sum;
        }
    }
}

/// Mix `Dirichlet(alpha)` noise into the root priors: `p <- (1-eps) p + eps * noise`.
fn apply_dirichlet_noise(edges: &mut [Edge], alpha: f32, epsilon: f32, rng: &mut AiRng) {
    if edges.is_empty() {
        return;
    }
    let mut noise: Vec<f64> = edges
        .iter()
        .map(|_| sample_gamma(alpha as f64, rng))
        .collect();
    let sum: f64 = noise.iter().sum();
    if sum <= 0.0 {
        return;
    }
    let eps = epsilon as f64;
    for (edge, n) in edges.iter_mut().zip(noise.iter_mut()) {
        *n /= sum;
        edge.prior = ((1.0 - eps) * edge.prior as f64 + eps * *n) as f32;
    }
}

/// Uniform sample in `[0, 1)` with 53 bits of precision.
fn next_unit_f64(rng: &mut AiRng) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

/// Standard normal via Box-Muller.
fn sample_normal(rng: &mut AiRng) -> f64 {
    let u1 = next_unit_f64(rng).max(1e-12);
    let u2 = next_unit_f64(rng);
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// Gamma(`alpha`, 1) via Marsaglia-Tsang (with the `alpha < 1` boosting trick).
fn sample_gamma(alpha: f64, rng: &mut AiRng) -> f64 {
    if alpha < 1.0 {
        let u = next_unit_f64(rng).max(1e-12);
        return sample_gamma(alpha + 1.0, rng) * u.powf(1.0 / alpha);
    }
    let d = alpha - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        let x = sample_normal(rng);
        let v = (1.0 + c * x).powi(3);
        if v <= 0.0 {
            continue;
        }
        let u = next_unit_f64(rng).max(1e-12);
        if u < 1.0 - 0.0331 * x.powi(4) {
            return d * v;
        }
        if u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
            return d * v;
        }
    }
}

fn pick_edge(edges: &[Edge], temperature: f32, rng: &mut AiRng) -> usize {
    if temperature <= 0.0 {
        return edges
            .iter()
            .enumerate()
            .max_by_key(|(_, e)| e.visits)
            .map(|(i, _)| i)
            .unwrap_or(0);
    }

    let inv = 1.0 / temperature as f64;
    let weights: Vec<f64> = edges.iter().map(|e| (e.visits as f64).powf(inv)).collect();
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return 0;
    }
    let mut draw = (rng.next_u64() as f64 / u64::MAX as f64) * total;
    for (i, weight) in weights.iter().enumerate() {
        draw -= weight;
        if draw <= 0.0 {
            return i;
        }
    }
    edges.len() - 1
}

/// Run MCTS from `state` and return the chosen action plus the visit-count policy target.
pub fn run_mcts(
    state: &TurnState,
    encoder: &Encoder,
    eval: &impl Evaluator,
    config: MctsConfig,
    rng: &mut AiRng,
) -> Option<MctsResult> {
    if state.winner.is_some() {
        return None;
    }

    let mut tree = Tree::new(encoder, eval, config.c_puct);
    let root = tree.push_node(state.clone());
    tree.expand(root);
    if tree.nodes[root].edges.is_empty() {
        return None;
    }

    if config.dirichlet_epsilon > 0.0 {
        apply_dirichlet_noise(
            &mut tree.nodes[root].edges,
            config.dirichlet_alpha,
            config.dirichlet_epsilon,
            rng,
        );
    }

    for _ in 0..config.simulations {
        tree.simulate(root);
    }

    let root_edges = &tree.nodes[root].edges;
    let total_visits: u32 = root_edges.iter().map(|e| e.visits).sum();

    let mut policy_target = vec![0.0f32; encoder.policy_len()];
    if total_visits > 0 {
        for edge in root_edges {
            policy_target[edge.index] = edge.visits as f32 / total_visits as f32;
        }
    }

    let value_sum: f64 = root_edges.iter().map(|e| e.value_sum).sum();
    let root_value = if total_visits > 0 {
        (value_sum / total_visits as f64) as f32
    } else {
        0.0
    };

    let chosen = pick_edge(root_edges, config.temperature, rng);
    Some(MctsResult {
        action: root_edges[chosen].action,
        policy_target,
        root_value,
    })
}

/// Pick the best legal action from the network's policy logits alone — a single inference,
/// no tree. The low-latency path for in-game play (the cooldown timer already paces turns).
pub fn policy_best_action(
    state: &TurnState,
    encoder: &Encoder,
    eval: &impl Evaluator,
) -> Option<GameAction> {
    if state.winner.is_some() {
        return None;
    }
    let (planes, k) = encoder.encode(state);
    let (logits, _value) = eval.evaluate(&planes);
    encoder
        .enumerate_legal_fast(state, k)
        .into_iter()
        .max_by(|(ia, _), (ib, _)| {
            logits[*ia]
                .partial_cmp(&logits[*ib])
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(_, action)| action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::AxialCoord;
    use crate::state::GameAction;

    fn uniform(encoder: &Encoder) -> UniformEvaluator {
        UniformEvaluator {
            policy_len: encoder.policy_len(),
        }
    }

    #[test]
    fn picks_immediate_winning_move() {
        // Player 0 (goal side 3 == q == -radius) one step from the goal.
        let mut state = TurnState::new(2, 3);
        state.current_player = 0;
        state.pawn_positions[0] = AxialCoord::new(-2, 0);
        state.pawn_positions[1] = AxialCoord::new(3, -1);

        let encoder = Encoder::new(3);
        let eval = uniform(&encoder);
        let mut rng = AiRng::seeded(1);
        let result = run_mcts(
            &state,
            &encoder,
            &eval,
            MctsConfig {
                simulations: 200,
                ..MctsConfig::default()
            },
            &mut rng,
        )
        .expect("has a move");

        // The winning move lands on the goal side.
        match result.action {
            GameAction::Move { target } => {
                assert!(target.is_on_side(3, 3), "should move onto the goal side");
            }
            other => panic!("expected a winning move, got {other:?}"),
        }
        // Root value should be strongly positive (a win is available).
        assert!(result.root_value > 0.5, "root value {}", result.root_value);
    }

    #[test]
    fn avoids_handing_opponent_the_win() {
        // It is player 0 to move, but player 1 is one step from winning (q == +radius is side 0,
        // player 1's goal). Player 0 cannot stop a pawn race in one ply, but the value should
        // reflect danger. Mainly: search runs and returns a legal action without panicking.
        let mut state = TurnState::new(2, 3);
        state.current_player = 0;
        state.pawn_positions[0] = AxialCoord::new(0, 0);
        state.pawn_positions[1] = AxialCoord::new(2, -1); // one step from side 0 (q == 3)

        let encoder = Encoder::new(3);
        let eval = uniform(&encoder);
        let mut rng = AiRng::seeded(2);
        let result =
            run_mcts(&state, &encoder, &eval, MctsConfig::default(), &mut rng).expect("has a move");

        // The returned action must be legal in the real state.
        let mut applied = state.clone();
        assert!(applied.try_apply_action(result.action).is_ok());
    }

    #[test]
    fn dirichlet_noise_keeps_search_valid() {
        let state = TurnState::new(2, 3);
        let encoder = Encoder::new(3);
        let eval = uniform(&encoder);
        let mut rng = AiRng::seeded(7);
        let result = run_mcts(
            &state,
            &encoder,
            &eval,
            MctsConfig {
                simulations: 64,
                temperature: 1.0,
                dirichlet_alpha: 0.3,
                dirichlet_epsilon: 0.25,
                ..MctsConfig::default()
            },
            &mut rng,
        )
        .expect("has a move");

        let sum: f32 = result.policy_target.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, "policy target sums to {sum}");
        let mut applied = state.clone();
        assert!(applied.try_apply_action(result.action).is_ok());
    }

    #[test]
    fn policy_best_action_returns_a_legal_move() {
        let state = TurnState::new(2, 3);
        let encoder = Encoder::new(3);
        let eval = uniform(&encoder);
        let action = policy_best_action(&state, &encoder, &eval).expect("an action");
        let mut applied = state.clone();
        assert!(applied.try_apply_action(action).is_ok());
    }

    #[test]
    fn policy_target_is_a_distribution_over_legal_actions() {
        let state = TurnState::new(2, 3);
        let encoder = Encoder::new(3);
        let eval = uniform(&encoder);
        let mut rng = AiRng::seeded(3);
        let result =
            run_mcts(&state, &encoder, &eval, MctsConfig::default(), &mut rng).expect("has a move");

        let sum: f32 = result.policy_target.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, "policy target sums to {sum}");
        assert_eq!(result.policy_target.len(), encoder.policy_len());
    }
}
