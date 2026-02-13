use bevy::prelude::*;

use crate::hex_grid::side_midpoint;

#[derive(Clone, Copy)]
pub struct PlayerDef {
    pub index: usize,
    pub pawn_color: Color,
    pub start_side: usize,
    pub goal_side: usize,
}

impl PlayerDef {
    pub fn start_coord(self) -> crate::hex_grid::AxialCoord {
        side_midpoint(self.start_side)
    }
}

pub fn three_players() -> [PlayerDef; 3] {
    [
        PlayerDef {
            index: 0,
            pawn_color: Color::srgb(0.92, 0.28, 0.24),
            start_side: 0,
            goal_side: 3,
        },
        PlayerDef {
            index: 1,
            pawn_color: Color::srgb(0.22, 0.56, 0.92),
            start_side: 1,
            goal_side: 4,
        },
        PlayerDef {
            index: 2,
            pawn_color: Color::srgb(0.95, 0.75, 0.2),
            start_side: 2,
            goal_side: 5,
        },
    ]
}
