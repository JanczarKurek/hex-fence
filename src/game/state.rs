use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::hex_grid::AxialCoord;

use super::player::{PlayerDef, three_players};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Resource)]
pub struct TurnState {
    pub players: [PlayerDef; 3],
    pub pawn_positions: [AxialCoord; 3],
    pub current_player: usize,
    pub winner: Option<usize>,
    pub fences_left: [usize; 3],
    pub blocked_edges: HashSet<EdgeKey>,
}

impl TurnState {
    pub fn new_three_players() -> Self {
        let players = three_players();
        let pawn_positions = [
            players[0].start_coord(),
            players[1].start_coord(),
            players[2].start_coord(),
        ];

        Self {
            players,
            pawn_positions,
            current_player: 0,
            winner: None,
            fences_left: [10; 3],
            blocked_edges: HashSet::new(),
        }
    }

    pub fn is_occupied(&self, coord: AxialCoord) -> bool {
        self.pawn_positions
            .into_iter()
            .any(|current| current == coord)
    }

    pub fn advance_turn(&mut self) {
        self.current_player = (self.current_player + 1) % self.players.len();
    }

    fn can_step(&self, from: AxialCoord, to: AxialCoord) -> bool {
        if !to.is_inside_board() {
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
                        .filter(|candidate| *candidate != from && self.can_step(occupied, *candidate))
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
            if !edge.a.is_inside_board() || !edge.b.is_inside_board() {
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
            if !has_path_to_goal(self.pawn_positions[index], player.goal_side, &future_blocked) {
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
}

fn has_path_to_goal(start: AxialCoord, goal_side: usize, blocked_edges: &HashSet<EdgeKey>) -> bool {
    let mut visited = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);

    while let Some(current) = queue.pop_front() {
        if current.is_on_side(goal_side) {
            return true;
        }

        for neighbor in current.neighbors() {
            if !neighbor.is_inside_board() {
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
