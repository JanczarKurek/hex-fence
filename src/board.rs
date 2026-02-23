use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::app_state::{AppPhase, GameConfig};
use crate::hex_grid::{AxialCoord, TILE_RADIUS};

pub struct BoardPlugin;

#[derive(Component)]
struct BoardTile;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppPhase::InGame), spawn_board)
            .add_systems(OnExit(AppPhase::InGame), cleanup_board);
    }
}

fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_config: Res<GameConfig>,
) {
    let tile_mesh = meshes.add(RegularPolygon::new(TILE_RADIUS, 6));
    let board_radius = game_config.board_radius;

    for q in -board_radius..=board_radius {
        for r in -board_radius..=board_radius {
            let coord = AxialCoord::new(q, r);
            if !coord.is_inside_board(board_radius) {
                continue;
            }

            let world_pos = coord.to_world();
            let color = tile_color(q, r);

            commands.spawn((
                BoardTile,
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

fn cleanup_board(mut commands: Commands, board_tiles: Query<Entity, With<BoardTile>>) {
    for entity in &board_tiles {
        commands.entity(entity).despawn();
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
