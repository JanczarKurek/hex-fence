use bevy::prelude::*;

pub use giereczka_core::hex::AxialCoord;

pub const TILE_RADIUS: f32 = 30.0;

/// Rendering helpers for hex coordinates. These live in the game crate (not
/// `giereczka-core`) because they pull in Bevy's `Vec2` and are purely about
/// drawing, not game rules. Bring this trait into scope to call `.to_world()`,
/// `.shade_index()`, or `AxialCoord::from_world(..)`.
pub trait HexRender: Sized {
    fn to_world(self) -> Vec2;
    fn shade_index(self) -> i32;
    fn from_world(world: Vec2) -> Self;
}

impl HexRender for AxialCoord {
    fn to_world(self) -> Vec2 {
        let q = self.q as f32;
        let r = self.r as f32;
        let x = TILE_RADIUS * (3.0_f32).sqrt() * (q + r / 2.0);
        let y = TILE_RADIUS * 1.5 * r;
        Vec2::new(x, y)
    }

    fn shade_index(self) -> i32 {
        (self.q - self.r).rem_euclid(3)
    }

    fn from_world(world: Vec2) -> Self {
        let q = ((3.0_f32).sqrt() / 3.0 * world.x - world.y / 3.0) / TILE_RADIUS;
        let r = ((2.0 / 3.0) * world.y) / TILE_RADIUS;
        axial_round(q, r)
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
