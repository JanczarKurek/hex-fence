//! Neural-network encoding: the shared contract between Rust self-play and Python training.
//!
//! Two things are defined here and MUST stay byte-identical with the Python side:
//!   1. The feature-plane tensor layout produced by [`Encoder::encode`].
//!   2. The fixed policy-head index map ([`Encoder::move_index`] / [`Encoder::fence_index`]).
//!
//! Everything is expressed from the **side-to-move's canonical perspective**: the board is
//! rotated so the current player always started on side 0 (goal side 3). For 2 players this
//! means one network serves both seats. See [`rotate`] and [`Encoder::canonical_k`].

use std::collections::{HashMap, HashSet, VecDeque};

use crate::fence_rules::{FenceShape, fence_edges};
use crate::hex::{AxialCoord, board_cells};
use crate::player::fences_per_player;
use crate::state::{EdgeKey, GameAction, TurnState, has_path_to_goal};

/// Number of feature planes per position.
///
/// Planes 0..=11 are the board occupancy / blocked-edge / fence-count / progress features;
/// planes 12 and 13 are the normalized BFS distance-to-goal fields for the side-to-move and
/// the opponent, computed over the *current* blocked edges (so fences reshape them). The
/// distance fields hand the network the racing landscape directly — the exact signal the
/// hand-written heuristic relies on and that cold-start self-play struggled to discover.
pub const PLANES: usize = 14;

/// Per-cell policy slots: 1 move-target + 4 shapes * 6 orientations of fences.
const FENCE_SLOTS_PER_CELL: usize = FenceShape::ALL.len() * 6; // 24

/// Rotate an axial coordinate by `k` steps of 60° (counter-clockwise).
///
/// One step maps direction index `d` to `d + 1`, so `k` steps map `d -> (d + k) % 6`.
/// Rotation is a bijection on the on-board cells of any radius.
pub fn rotate(coord: AxialCoord, k: usize) -> AxialCoord {
    let mut q = coord.q;
    let mut r = coord.r;
    for _ in 0..(k % 6) {
        // 60° step in axial coords: (q, r) -> (q + r, -q).
        let nq = q + r;
        let nr = -q;
        q = nq;
        r = nr;
    }
    AxialCoord::new(q, r)
}

/// Direction index after rotating the board by `k` steps.
pub fn rotate_dir(direction: usize, k: usize) -> usize {
    (direction + k) % 6
}

/// Board side index after rotating the board by `k` steps.
///
/// Unlike directions, the side indexing of [`AxialCoord::is_on_side`] is not linear under
/// rotation: a single 60° step follows the cycle `0 -> 4 -> 2 -> 3 -> 1 -> 5 -> 0` (opposite
/// sides `s` and `s + 3` always stay opposite). `STEP[s]` is the side after one step.
pub fn rotate_side(side: usize, k: usize) -> usize {
    const STEP: [usize; 6] = [4, 5, 3, 1, 2, 0];
    let mut side = side % 6;
    for _ in 0..(k % 6) {
        side = STEP[side];
    }
    side
}

/// A geometrically valid fence placement (state-independent): distinct, on-board, adjacent
/// edges. Precomputed once per radius so enumeration only does the cheap state-dependent
/// checks (not already blocked, connectivity).
struct StaticFence {
    anchor: AxialCoord,
    shape: FenceShape,
    orientation: usize,
    edges: [EdgeKey; 3],
}

/// A canonical encoder for a fixed board radius. Precomputes the deterministic cell
/// ordering used by the policy index map and the geometrically valid fence placements.
pub struct Encoder {
    radius: i32,
    cells: Vec<AxialCoord>,
    index: HashMap<AxialCoord, usize>,
    fences: Vec<StaticFence>,
}

impl Encoder {
    pub fn new(radius: i32) -> Self {
        let cells = board_cells(radius);
        let index = cells
            .iter()
            .enumerate()
            .map(|(i, c)| (*c, i))
            .collect::<HashMap<_, _>>();

        let mut fences = Vec::new();
        for &anchor in &cells {
            for shape in FenceShape::ALL {
                for orientation in 0..6 {
                    let edges = fence_edges(anchor, shape, orientation);
                    if fence_geometry_static_valid(&edges, radius) {
                        fences.push(StaticFence {
                            anchor,
                            shape,
                            orientation,
                            edges,
                        });
                    }
                }
            }
        }

        Self {
            radius,
            cells,
            index,
            fences,
        }
    }

    pub fn radius(&self) -> i32 {
        self.radius
    }

    /// Number of on-board cells (the move-block length).
    pub fn n_cells(&self) -> usize {
        self.cells.len()
    }

    /// Side length of the square plane tensor (`2R + 1`).
    pub fn dim(&self) -> usize {
        (2 * self.radius + 1) as usize
    }

    /// Total policy-head length: `N + N * 24`.
    pub fn policy_len(&self) -> usize {
        self.n_cells() * (1 + FENCE_SLOTS_PER_CELL)
    }

    /// Canonical index in `0..N` of a cell, or `None` if off-board.
    pub fn cell_index(&self, coord: AxialCoord) -> Option<usize> {
        self.index.get(&coord).copied()
    }

    /// Rotation `k` that maps the side-to-move's start side to side 0 (goal side to 3).
    pub fn canonical_k(&self, state: &TurnState) -> usize {
        let start_side = state.players[state.current_player].start_side % 6;
        (0..6)
            .find(|&k| rotate_side(start_side, k) == 0)
            .unwrap_or(0)
    }

    /// Policy index of a move to `canonical_target` (already in canonical frame).
    pub fn move_index(&self, canonical_target: AxialCoord) -> Option<usize> {
        self.cell_index(canonical_target)
    }

    /// Policy index of a fence anchored at `canonical_anchor` (canonical frame).
    pub fn fence_index(
        &self,
        canonical_anchor: AxialCoord,
        shape: FenceShape,
        canonical_orientation: usize,
    ) -> Option<usize> {
        let anchor_ci = self.cell_index(canonical_anchor)?;
        Some(
            self.n_cells()
                + anchor_ci * FENCE_SLOTS_PER_CELL
                + shape.to_index() * 6
                + canonical_orientation % 6,
        )
    }

    /// Decode a canonical policy index back into an original-frame [`GameAction`],
    /// given the rotation `k` that was applied during encoding.
    pub fn decode_index(&self, index: usize, k: usize) -> Option<GameAction> {
        let n = self.n_cells();
        let inverse = (6 - (k % 6)) % 6;
        if index < n {
            let canonical_target = self.cells[index];
            Some(GameAction::Move {
                target: rotate(canonical_target, inverse),
            })
        } else {
            let fence = index - n;
            let anchor_ci = fence / FENCE_SLOTS_PER_CELL;
            let rem = fence % FENCE_SLOTS_PER_CELL;
            let shape = FenceShape::from_index(rem / 6)?;
            let canonical_orientation = rem % 6;
            let canonical_anchor = *self.cells.get(anchor_ci)?;
            let anchor = rotate(canonical_anchor, inverse);
            let orientation = (canonical_orientation + inverse) % 6;
            Some(GameAction::PlaceFence {
                edges: fence_edges(anchor, shape, orientation),
            })
        }
    }

    /// Encode a position into canonical feature planes. Returns the flat tensor
    /// (`PLANES * dim * dim`, channel-major) and the rotation `k` that was applied.
    pub fn encode(&self, state: &TurnState) -> (Vec<f32>, usize) {
        let k = self.canonical_k(state);
        let dim = self.dim();
        let plane_size = dim * dim;
        let mut planes = vec![0.0f32; PLANES * plane_size];

        let radius = self.radius;
        let mut put = |ch: usize, coord: AxialCoord, value: f32| {
            let rotated = rotate(coord, k);
            let row = (rotated.r + radius) as usize;
            let col = (rotated.q + radius) as usize;
            planes[ch * plane_size + row * dim + col] = value;
        };

        let cur = state.current_player;
        let opp = (cur + 1) % state.players.len();

        // 0: side-to-move pawn, 1: opponent pawn.
        put(0, state.pawn_positions[cur], 1.0);
        put(1, state.pawn_positions[opp], 1.0);

        // 2: on-board mask (rotation maps the board onto itself, so this fills every cell).
        for &cell in &self.cells {
            put(2, cell, 1.0);
        }

        // 3..=8: blocked-edge planes, one per canonical direction, both endpoints lit.
        for edge in &state.blocked_edges {
            if let Some(d) = edge.a.direction_to(edge.b) {
                put(3 + rotate_dir(d, k), edge.a, 1.0);
                put(3 + rotate_dir((d + 3) % 6, k), edge.b, 1.0);
            }
        }

        // 9: self fences-left, 10: opponent fences-left (normalized), 11: progress.
        let max_fences = fences_per_player(state.players.len()).max(1) as f32;
        let self_fences = state.fences_left[cur] as f32 / max_fences;
        let opp_fences = state.fences_left[opp] as f32 / max_fences;
        let total_capacity = max_fences * state.players.len() as f32;
        let placed: usize = state
            .fences_left
            .iter()
            .map(|left| fences_per_player(state.players.len()).saturating_sub(*left))
            .sum();
        let progress = placed as f32 / total_capacity.max(1.0);
        for &cell in &self.cells {
            put(9, cell, self_fences);
            put(10, cell, opp_fences);
            put(11, cell, progress);
        }

        // 12: distance-to-own-goal field, 13: distance-to-opponent-goal field (normalized).
        // Multi-source BFS from each goal side over the current blocked edges, so a freshly
        // placed fence immediately shows up as a longer path for whoever it hinders.
        let self_goal = state.players[cur].goal_side;
        let opp_goal = state.players[opp].goal_side;
        let self_field = distance_field(self_goal, radius, &state.blocked_edges);
        let opp_field = distance_field(opp_goal, radius, &state.blocked_edges);
        let norm = (2 * radius).max(1) as f32; // longest possible board-graph distance
        let normalize = |d: Option<&u32>| d.map_or(1.0, |&d| (d as f32 / norm).min(1.0));
        for &cell in &self.cells {
            put(12, cell, normalize(self_field.get(&cell)));
            put(13, cell, normalize(opp_field.get(&cell)));
        }

        (planes, k)
    }

    /// All legal actions for the side-to-move, paired with their canonical policy index
    /// and the original-frame [`GameAction`] to apply.
    ///
    /// This is the simple reference enumerator: it tests every `(anchor, shape, orientation)`
    /// against [`TurnState::can_place_fence`] (a full BFS per candidate). [`enumerate_legal_fast`]
    /// produces the identical set far faster; this one exists to fuzz-check it.
    ///
    /// [`enumerate_legal_fast`]: Encoder::enumerate_legal_fast
    pub fn enumerate_legal(&self, state: &TurnState, k: usize) -> Vec<(usize, GameAction)> {
        let mut out = Vec::new();

        for target in state.legal_moves_for_current() {
            if let Some(idx) = self.move_index(rotate(target, k)) {
                out.push((idx, GameAction::Move { target }));
            }
        }

        if state.fences_left[state.current_player] > 0 {
            for &anchor in &self.cells {
                for shape in FenceShape::ALL {
                    for orientation in 0..6 {
                        let edges = fence_edges(anchor, shape, orientation);
                        if !state.can_place_fence(&edges) {
                            continue;
                        }
                        if let Some(idx) =
                            self.fence_index(rotate(anchor, k), shape, rotate_dir(orientation, k))
                        {
                            out.push((idx, GameAction::PlaceFence { edges }));
                        }
                    }
                }
            }
        }

        out
    }

    /// Same result as [`enumerate_legal`](Encoder::enumerate_legal), but avoids the per-fence
    /// connectivity BFS in the common case.
    ///
    /// A fence can only wall a player off if one of its three edges lies on that player's
    /// *current* shortest path — otherwise the old path is still an unblocked witness. So we
    /// precompute each player's shortest-path edge set once and only run a real BFS for the
    /// rare candidates that intersect it.
    pub fn enumerate_legal_fast(&self, state: &TurnState, k: usize) -> Vec<(usize, GameAction)> {
        let mut out = Vec::new();

        for target in state.legal_moves_for_current() {
            if let Some(idx) = self.move_index(rotate(target, k)) {
                out.push((idx, GameAction::Move { target }));
            }
        }

        if state.fences_left[state.current_player] == 0 {
            return out;
        }

        let blocked = &state.blocked_edges;
        let radius = state.board_radius;
        let player_paths: Vec<HashSet<EdgeKey>> = state
            .players
            .iter()
            .enumerate()
            .map(|(i, player)| {
                shortest_path_edges(state.pawn_positions[i], player.goal_side, radius, blocked)
            })
            .collect();

        for fence in &self.fences {
            // Already-blocked edges make the placement illegal.
            if fence.edges.iter().any(|edge| blocked.contains(edge)) {
                continue;
            }

            // Only the players whose shortest path an edge touches can be cut off;
            // verify those with a real BFS over the would-be-blocked graph.
            let mut future_blocked: Option<HashSet<EdgeKey>> = None;
            let mut connected = true;
            for (i, player) in state.players.iter().enumerate() {
                if !fence
                    .edges
                    .iter()
                    .any(|edge| player_paths[i].contains(edge))
                {
                    continue;
                }
                let blocked_now = future_blocked.get_or_insert_with(|| {
                    let mut set = blocked.clone();
                    set.extend(fence.edges);
                    set
                });
                if !has_path_to_goal(
                    state.pawn_positions[i],
                    player.goal_side,
                    radius,
                    blocked_now,
                ) {
                    connected = false;
                    break;
                }
            }

            if connected
                && let Some(idx) = self.fence_index(
                    rotate(fence.anchor, k),
                    fence.shape,
                    rotate_dir(fence.orientation, k),
                )
            {
                out.push((idx, GameAction::PlaceFence { edges: fence.edges }));
            }
        }

        out
    }
}

/// State-independent geometry validity for a fence: three distinct, on-board, adjacent edges.
fn fence_geometry_static_valid(edges: &[EdgeKey; 3], radius: i32) -> bool {
    if edges[0] == edges[1] || edges[0] == edges[2] || edges[1] == edges[2] {
        return false;
    }
    for edge in edges {
        if !edge.a.is_inside_board(radius) || !edge.b.is_inside_board(radius) {
            return false;
        }
        if edge.a.direction_to(edge.b).is_none() {
            return false;
        }
    }
    true
}

/// BFS distance from every reachable cell to `goal_side`, over the current `blocked` edges.
///
/// Multi-source from all goal-side cells outward; since the move graph is undirected, the
/// distance found equals each cell's shortest unblocked path length to that side. Cells with
/// no entry are walled off from the side (left to the caller to treat as max distance).
fn distance_field(
    goal_side: usize,
    radius: i32,
    blocked: &HashSet<EdgeKey>,
) -> HashMap<AxialCoord, u32> {
    let mut dist: HashMap<AxialCoord, u32> = HashMap::new();
    let mut queue: VecDeque<AxialCoord> = VecDeque::new();
    for cell in board_cells(radius) {
        if cell.is_on_side(goal_side, radius) {
            dist.insert(cell, 0);
            queue.push_back(cell);
        }
    }

    while let Some(current) = queue.pop_front() {
        let d = dist[&current];
        for neighbor in current.neighbors() {
            if !neighbor.is_inside_board(radius) {
                continue;
            }
            if blocked.contains(&EdgeKey::from_cells(current, neighbor)) {
                continue;
            }
            if dist.contains_key(&neighbor) {
                continue;
            }
            dist.insert(neighbor, d + 1);
            queue.push_back(neighbor);
        }
    }

    dist
}

/// Edges along one shortest path from `start` to `goal_side` (empty if already on the side).
fn shortest_path_edges(
    start: AxialCoord,
    goal_side: usize,
    radius: i32,
    blocked: &HashSet<EdgeKey>,
) -> HashSet<EdgeKey> {
    let mut edges = HashSet::new();
    if start.is_on_side(goal_side, radius) {
        return edges;
    }

    let mut predecessor: HashMap<AxialCoord, AxialCoord> = HashMap::new();
    let mut visited: HashSet<AxialCoord> = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    let mut goal_cell = None;

    'search: while let Some(current) = queue.pop_front() {
        for neighbor in current.neighbors() {
            if !neighbor.is_inside_board(radius) {
                continue;
            }
            if blocked.contains(&EdgeKey::from_cells(current, neighbor)) {
                continue;
            }
            if !visited.insert(neighbor) {
                continue;
            }
            predecessor.insert(neighbor, current);
            if neighbor.is_on_side(goal_side, radius) {
                goal_cell = Some(neighbor);
                break 'search;
            }
            queue.push_back(neighbor);
        }
    }

    let mut cell = goal_cell;
    while let Some(current) = cell {
        if let Some(&prev) = predecessor.get(&current) {
            edges.insert(EdgeKey::from_cells(prev, current));
            cell = Some(prev);
        } else {
            break;
        }
    }

    edges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::board_cells as cells_for;
    use crate::state::EdgeKey;

    #[test]
    fn policy_len_matches_plan() {
        assert_eq!(Encoder::new(3).policy_len(), 925);
        assert_eq!(Encoder::new(4).policy_len(), 1525);
    }

    #[test]
    fn rotate_matches_direction_shift() {
        let origin = AxialCoord::new(0, 0);
        for k in 0..6 {
            for d in 0..6 {
                assert_eq!(
                    rotate(origin.neighbor_in_direction(d), k),
                    origin.neighbor_in_direction((d + k) % 6),
                    "dir {d} rotated by {k}"
                );
            }
        }
    }

    #[test]
    fn rotate_round_trips_and_preserves_board() {
        for radius in [3, 4] {
            for &cell in &cells_for(radius) {
                for k in 0..6 {
                    let rotated = rotate(cell, k);
                    assert!(rotated.is_inside_board(radius), "rotation left the board");
                    assert_eq!(rotate(rotated, (6 - k) % 6), cell, "round trip failed");
                }
            }
        }
    }

    #[test]
    fn rotate_preserves_sides() {
        let radius = 4;
        for &cell in &cells_for(radius) {
            for side in 0..6 {
                for k in 0..6 {
                    assert_eq!(
                        cell.is_on_side(side, radius),
                        rotate(cell, k).is_on_side(rotate_side(side, k), radius),
                    );
                }
            }
        }
    }

    #[test]
    fn encode_has_expected_shape() {
        let encoder = Encoder::new(3);
        let state = TurnState::new(2, 3);
        let (planes, _k) = encoder.encode(&state);
        assert_eq!(planes.len(), PLANES * encoder.dim() * encoder.dim());
    }

    #[test]
    fn canonicalization_makes_both_seats_symmetric() {
        // In the initial 2-player position the two pawns are mirror images. From each
        // player's canonical perspective the self-pawn must land on the same cell.
        let encoder = Encoder::new(3);
        let plane_size = encoder.dim() * encoder.dim();

        let mut state = TurnState::new(2, 3);
        let (planes_p0, _) = encoder.encode(&state);

        state.current_player = 1;
        let (planes_p1, _) = encoder.encode(&state);

        // Plane 0 is the self-pawn; the canonical self-pawn cell is identical for both seats.
        assert_eq!(
            &planes_p0[0..plane_size],
            &planes_p1[0..plane_size],
            "self-pawn plane differs between canonical perspectives"
        );
    }

    #[test]
    fn legal_action_indices_round_trip_through_decode() {
        // Exercise a few positions for both seats and confirm index <-> action is a bijection.
        for player_to_move in [0usize, 1] {
            let mut state = TurnState::new(2, 3);
            state.current_player = player_to_move;
            let encoder = Encoder::new(3);
            let k = encoder.canonical_k(&state);

            let legal = encoder.enumerate_legal(&state, k);
            assert!(!legal.is_empty());
            for (idx, action) in legal {
                assert!(idx < encoder.policy_len(), "index out of range");
                let decoded = encoder.decode_index(idx, k).expect("decodes");
                match (action, decoded) {
                    (GameAction::Move { target: a }, GameAction::Move { target: b }) => {
                        assert_eq!(a, b, "move target round trip");
                    }
                    (GameAction::PlaceFence { edges: a }, GameAction::PlaceFence { edges: b }) => {
                        let set_a: std::collections::HashSet<EdgeKey> = a.into_iter().collect();
                        let set_b: std::collections::HashSet<EdgeKey> = b.into_iter().collect();
                        assert_eq!(set_a, set_b, "fence edges round trip");
                    }
                    (a, b) => panic!("action kind changed: {a:?} -> {b:?}"),
                }
            }
        }
    }

    #[test]
    fn fast_enumerator_matches_reference() {
        use crate::heuristic::AiRng;
        let encoder = Encoder::new(3);
        let mut rng = AiRng::seeded(0xC0FFEE);

        for game in 0..8 {
            let mut state = TurnState::new(2, 3);
            let mut step = 0;
            while state.winner.is_none() && step < 30 {
                let k = encoder.canonical_k(&state);
                let mut reference: Vec<usize> = encoder
                    .enumerate_legal(&state, k)
                    .into_iter()
                    .map(|(idx, _)| idx)
                    .collect();
                let mut fast: Vec<usize> = encoder
                    .enumerate_legal_fast(&state, k)
                    .into_iter()
                    .map(|(idx, _)| idx)
                    .collect();
                reference.sort_unstable();
                fast.sort_unstable();
                assert_eq!(reference, fast, "game {game} step {step}");

                let legal = encoder.enumerate_legal_fast(&state, k);
                if legal.is_empty() {
                    break;
                }
                let (_, action) = legal[rng.choose_index(legal.len())];
                state.try_apply_action(action).expect("legal");
                step += 1;
            }
        }
    }

    #[test]
    fn fence_indices_are_unique_per_tuple() {
        let encoder = Encoder::new(3);
        let mut seen = std::collections::HashSet::new();
        for &anchor in &cells_for(3) {
            for shape in FenceShape::ALL {
                for orientation in 0..6 {
                    let idx = encoder.fence_index(anchor, shape, orientation).unwrap();
                    assert!(idx >= encoder.n_cells());
                    assert!(idx < encoder.policy_len());
                    assert!(seen.insert(idx), "duplicate fence index");
                }
            }
        }
        assert_eq!(seen.len(), encoder.n_cells() * 24);
    }
}
