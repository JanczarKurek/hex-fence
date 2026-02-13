use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::hex_grid::{AxialCoord, BOARD_RADIUS, TILE_RADIUS};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_board);
    }
}

fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tile_mesh = meshes.add(RegularPolygon::new(TILE_RADIUS, 6));

    for q in -BOARD_RADIUS..=BOARD_RADIUS {
        for r in -BOARD_RADIUS..=BOARD_RADIUS {
            let coord = AxialCoord::new(q, r);
            if !coord.is_inside_board() {
                continue;
            }

            let world_pos = coord.to_world();
            let color = tile_color(q, r);

            commands.spawn((
                Mesh2d(tile_mesh.clone()),
                MeshMaterial2d(materials.add(color)),
                Transform {
                    translation: Vec3::new(world_pos.x, world_pos.y, 0.0),
                    ..default()
                },
            ));
        }
    }
}

fn tile_color(q: i32, r: i32) -> Color {
    // A 3-coloring of axial coordinates. For every hex neighbor step,
    // `(q - r) mod 3` changes, so adjacent tiles always differ.
    match (q - r).rem_euclid(3) {
        0 => Color::srgb(0.25, 0.55, 0.35),
        1 => Color::srgb(0.21, 0.48, 0.31),
        _ => Color::srgb(0.17, 0.42, 0.27),
    }
}
