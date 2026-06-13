use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::app_state::GameConfig;
use crate::hex_grid::{HexRender, TILE_RADIUS};

use super::components::Pawn;
use super::state::{GameState, TurnState};

pub fn spawn_pawn_entities(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    turn_state: &TurnState,
    game_config: &GameConfig,
) {
    let pawn_mesh = meshes.add(RegularPolygon::new(TILE_RADIUS * 0.45, 24));

    for (player, position) in turn_state.players.iter().zip(&turn_state.pawn_positions) {
        let world = position.to_world();

        commands.spawn((
            Pawn {
                player_index: player.index,
            },
            Mesh2d(pawn_mesh.clone()),
            MeshMaterial2d(materials.add(game_config.pawn_color(player.index))),
            Transform::from_xyz(world.x, world.y, 2.0),
        ));
    }
}

pub fn spawn_pawns(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    turn_state: Res<GameState>,
    game_config: Res<GameConfig>,
) {
    spawn_pawn_entities(
        &mut commands,
        &mut meshes,
        &mut materials,
        &turn_state.0,
        &game_config,
    );
}
