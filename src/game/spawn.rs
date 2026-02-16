use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::hex_grid::TILE_RADIUS;

use super::components::Pawn;
use super::state::TurnState;

pub fn spawn_pawns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    turn_state: Res<TurnState>,
) {
    let pawn_mesh = meshes.add(RegularPolygon::new(TILE_RADIUS * 0.45, 24));

    for player in &turn_state.players {
        let start = player.start_coord(turn_state.board_radius);
        let world = start.to_world();

        commands.spawn((
            Pawn {
                player_index: player.index,
            },
            Mesh2d(pawn_mesh.clone()),
            MeshMaterial2d(materials.add(player.pawn_color)),
            Transform::from_xyz(world.x, world.y, 2.0),
        ));
    }
}
