use bevy::prelude::*;

pub const BOARD_RADIUS: i32 = 4;
pub const TILE_RADIUS: f32 = 30.0;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AxialCoord {
    pub q: i32,
    pub r: i32,
}

impl AxialCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn is_inside_board(self) -> bool {
        let s = -self.q - self.r;
        self.q.abs() <= BOARD_RADIUS && self.r.abs() <= BOARD_RADIUS && s.abs() <= BOARD_RADIUS
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
        self.neighbors().iter().position(|neighbor| *neighbor == other)
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

    pub fn is_on_side(self, side: usize) -> bool {
        match side % 6 {
            0 => self.q == BOARD_RADIUS,
            1 => self.r == BOARD_RADIUS,
            2 => self.q + self.r == -BOARD_RADIUS,
            3 => self.q == -BOARD_RADIUS,
            4 => self.r == -BOARD_RADIUS,
            _ => self.q + self.r == BOARD_RADIUS,
        }
    }
}

pub fn side_midpoint(side: usize) -> AxialCoord {
    let mid = BOARD_RADIUS / 2;
    match side % 6 {
        0 => AxialCoord::new(BOARD_RADIUS, -mid),
        1 => AxialCoord::new(-mid, BOARD_RADIUS),
        2 => AxialCoord::new(-mid, -mid),
        3 => AxialCoord::new(-BOARD_RADIUS, mid),
        4 => AxialCoord::new(mid, -BOARD_RADIUS),
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
