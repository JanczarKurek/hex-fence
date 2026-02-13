use bevy::prelude::*;

use crate::hex_grid::AxialCoord;

use super::player::{PlayerDef, three_players};

#[derive(Resource)]
pub struct TurnState {
    pub players: [PlayerDef; 3],
    pub pawn_positions: [AxialCoord; 3],
    pub current_player: usize,
    pub winner: Option<usize>,
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

    pub fn legal_moves_for_current(&self) -> Vec<AxialCoord> {
        let current_pos = self.pawn_positions[self.current_player];
        let mut legal_moves = Vec::new();

        for neighbor in current_pos.neighbors() {
            if !neighbor.is_inside_board() {
                continue;
            }

            if !self.is_occupied(neighbor) {
                legal_moves.push(neighbor);
                continue;
            }

            let dq = neighbor.q - current_pos.q;
            let dr = neighbor.r - current_pos.r;
            let jump_target = AxialCoord::new(neighbor.q + dq, neighbor.r + dr);

            if jump_target.is_inside_board() && !self.is_occupied(jump_target) {
                legal_moves.push(jump_target);
            }
        }

        legal_moves
    }
}
