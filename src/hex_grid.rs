use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const TILE_RADIUS: f32 = 30.0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AxialCoord {
    pub q: i32,
    pub r: i32,
}

impl AxialCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn is_inside_board(self, board_radius: i32) -> bool {
        let s = -self.q - self.r;
        self.q.abs() <= board_radius && self.r.abs() <= board_radius && s.abs() <= board_radius
    }

    pub fn neighbors(self) -> [Self; 6] {
        [
            Self::new(self.q + 1, self.r),
            Self::new(self.q + 1, self.r - 1),
            Self::new(self.q, self.r - 1),
            Self::new(self.q - 1, self.r),
            Self::new(self.q - 1, self.r + 1),
            Self::new(self.q, self.r + 1),
        ]
    }

    pub fn neighbor_in_direction(self, direction: usize) -> Self {
        self.neighbors()[direction % 6]
    }

    pub fn direction_to(self, other: Self) -> Option<usize> {
        self.neighbors()
            .iter()
            .position(|neighbor| *neighbor == other)
    }

    pub fn to_world(self) -> Vec2 {
        let q = self.q as f32;
        let r = self.r as f32;
        let x = TILE_RADIUS * (3.0_f32).sqrt() * (q + r / 2.0);
        let y = TILE_RADIUS * 1.5 * r;
        Vec2::new(x, y)
    }

    pub fn from_world(world: Vec2) -> Self {
        let q = ((3.0_f32).sqrt() / 3.0 * world.x - world.y / 3.0) / TILE_RADIUS;
        let r = ((2.0 / 3.0) * world.y) / TILE_RADIUS;
        axial_round(q, r)
    }

    pub fn is_on_side(self, side: usize, board_radius: i32) -> bool {
        match side % 6 {
            0 => self.q == board_radius,
            1 => self.r == board_radius,
            2 => self.q + self.r == -board_radius,
            3 => self.q == -board_radius,
            4 => self.r == -board_radius,
            _ => self.q + self.r == board_radius,
        }
    }
}

pub fn side_midpoint(side: usize, board_radius: i32) -> AxialCoord {
    let mid = board_radius / 2;
    match side % 6 {
        0 => AxialCoord::new(board_radius, -mid),
        1 => AxialCoord::new(-mid, board_radius),
        2 => AxialCoord::new(-mid, -mid),
        3 => AxialCoord::new(-board_radius, mid),
        4 => AxialCoord::new(mid, -board_radius),
        _ => AxialCoord::new(mid, mid),
    }
}

fn axial_round(qf: f32, rf: f32) -> AxialCoord {
    let sf = -qf - rf;

    let mut q = qf.round();
    let mut r = rf.round();
    let s = sf.round();

    let q_diff = (q - qf).abs();
    let r_diff = (r - rf).abs();
    let s_diff = (s - sf).abs();

    if q_diff > r_diff && q_diff > s_diff {
        q = -r - s;
    } else if r_diff > s_diff {
        r = -q - s;
    }

    AxialCoord::new(q as i32, r as i32)
}
