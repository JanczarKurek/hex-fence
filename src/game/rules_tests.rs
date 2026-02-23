use std::collections::HashSet;

use crate::hex_grid::AxialCoord;

use super::fence::{FenceShape, fence_edges};
use super::state::{ActionError, EdgeKey, TurnState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestAction {
    Move(AxialCoord),
    PlaceFence([EdgeKey; 3]),
}

#[derive(Debug, PartialEq, Eq)]
struct GameSnapshot {
    current_player: usize,
    winner: Option<usize>,
    pawn_positions: Vec<AxialCoord>,
    fences_left: Vec<usize>,
    blocked_edges: HashSet<EdgeKey>,
}

impl From<&TurnState> for GameSnapshot {
    fn from(state: &TurnState) -> Self {
        Self {
            current_player: state.current_player,
            winner: state.winner,
            pawn_positions: state.pawn_positions.clone(),
            fences_left: state.fences_left.clone(),
            blocked_edges: state.blocked_edges.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct SequenceError {
    step: usize,
    action: TestAction,
    error: ActionError,
}

fn run_sequence(
    mut state: TurnState,
    actions: &[TestAction],
) -> Result<GameSnapshot, SequenceError> {
    for (step, action) in actions.iter().copied().enumerate() {
        let result = match action {
            TestAction::Move(target) => state.try_move_current_pawn(target).map(|_| ()),
            TestAction::PlaceFence(edges) => state.try_place_fence(&edges),
        };

        if let Err(error) = result {
            return Err(SequenceError {
                step,
                action,
                error,
            });
        }
    }

    Ok(GameSnapshot::from(&state))
}

#[test]
fn sequence_moves_pawns_and_matches_expected_snapshot() {
    let initial = TurnState::new(2, 3);
    let actions = [
        TestAction::Move(AxialCoord::new(2, -1)),
        TestAction::Move(AxialCoord::new(-2, 1)),
    ];

    let expected = GameSnapshot {
        current_player: 0,
        winner: None,
        pawn_positions: vec![AxialCoord::new(2, -1), AxialCoord::new(-2, 1)],
        fences_left: vec![10, 10],
        blocked_edges: HashSet::new(),
    };

    let actual = run_sequence(initial, &actions).expect("sequence should be valid");
    assert_eq!(actual, expected);
}

#[test]
fn sequence_places_fence_and_updates_state() {
    let initial = TurnState::new(2, 3);
    let edges = fence_edges(AxialCoord::new(0, 0), FenceShape::C, 0);
    let actions = [TestAction::PlaceFence(edges)];

    let mut expected_edges = HashSet::new();
    expected_edges.extend(edges);
    let expected = GameSnapshot {
        current_player: 1,
        winner: None,
        pawn_positions: vec![AxialCoord::new(3, -1), AxialCoord::new(-3, 1)],
        fences_left: vec![9, 10],
        blocked_edges: expected_edges,
    };

    let actual = run_sequence(initial, &actions).expect("fence placement should be valid");
    assert_eq!(actual, expected);
}

#[test]
fn sequence_reports_first_invalid_step() {
    let initial = TurnState::new(2, 3);
    let actions = [TestAction::Move(AxialCoord::new(0, 0))];

    let error = run_sequence(initial, &actions).expect_err("sequence should fail on invalid move");
    assert_eq!(
        error,
        SequenceError {
            step: 0,
            action: TestAction::Move(AxialCoord::new(0, 0)),
            error: ActionError::IllegalMove,
        }
    );
}

#[test]
fn s_and_mirrored_s_are_distinct_shapes() {
    let anchor = AxialCoord::new(0, 0);
    let s_edges = fence_edges(anchor, FenceShape::S, 0);
    let mirrored_edges = fence_edges(anchor, FenceShape::SMirrored, 0);

    assert_ne!(s_edges, mirrored_edges);
}

#[test]
fn mirrored_s_matches_expected_connected_pattern_for_orientation_zero() {
    let anchor = AxialCoord::new(0, 0);
    let actual: HashSet<_> = fence_edges(anchor, FenceShape::SMirrored, 0)
        .into_iter()
        .collect();

    let n0 = anchor.neighbor_in_direction(0);
    let n1 = anchor.neighbor_in_direction(1);
    let expected_next = n0.neighbor_in_direction(4);
    let expected = HashSet::from([
        EdgeKey::from_cells(anchor, n0),
        EdgeKey::from_cells(anchor, n1),
        EdgeKey::from_cells(n0, expected_next),
    ]);

    assert_eq!(actual, expected);
}
