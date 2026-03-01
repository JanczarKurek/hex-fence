use bevy::prelude::*;

use crate::app_state::PlayerColor;
use crate::hex_grid::side_midpoint;

#[derive(Clone, Copy)]
pub struct PlayerDef {
    pub index: usize,
    pub pawn_color: Color,
    pub start_side: usize,
    pub goal_side: usize,
}

impl PlayerDef {
    pub fn start_coord(self, board_radius: i32) -> crate::hex_grid::AxialCoord {
        side_midpoint(self.start_side, board_radius)
    }
}

pub fn players_for_count(player_count: usize, player_colors: &[PlayerColor; 6]) -> Vec<PlayerDef> {
    let start_sides: Vec<usize> = match player_count {
        2 => vec![0, 3],
        3 => vec![0, 1, 2],
        6 => vec![0, 1, 2, 3, 4, 5],
        _ => vec![0, 2, 4],
    };

    start_sides
        .iter()
        .enumerate()
        .map(|(index, side)| PlayerDef {
            index,
            pawn_color: player_colors[index % player_colors.len()].color(),
            start_side: *side,
            goal_side: (side + 3) % 6,
        })
        .collect()
}

pub fn fences_per_player(player_count: usize) -> usize {
    match player_count {
        2 => 10,
        3 => 8,
        6 => 5,
        _ => 8,
    }
}
