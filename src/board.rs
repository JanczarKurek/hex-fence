use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::app_state::{AppPhase, GameConfig};
use crate::game::{HoveredGoalPreview, state::TurnState};
use crate::hex_grid::{AxialCoord, TILE_RADIUS};

pub struct BoardPlugin;

#[derive(Component)]
struct BoardTile {
    coord: AxialCoord,
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppPhase::InGame), spawn_board)
            .add_systems(OnExit(AppPhase::InGame), cleanup_board)
            .add_systems(Update, update_goal_preview.run_if(in_state(AppPhase::InGame)));
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
            let color = normal_tile_color((q - r).rem_euclid(3));

            commands.spawn((
                BoardTile { coord },
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

fn update_goal_preview(
    hovered_preview: Res<HoveredGoalPreview>,
    turn_state: Res<TurnState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_tiles: Query<(&BoardTile, &MeshMaterial2d<ColorMaterial>)>,
) {
    if !hovered_preview.is_changed() && !turn_state.is_changed() {
        return;
    }

    let Some(player_index) = hovered_preview.player_index else {
        for (tile, material_handle) in &board_tiles {
            if let Some(material) = materials.get_mut(material_handle) {
                let shade_index = (tile.coord.q - tile.coord.r).rem_euclid(3);
                material.color = normal_tile_color(shade_index);
            }
        }
        return;
    };

    let Some(player) = turn_state.players.get(player_index) else {
        return;
    };

    let goal_side = player.goal_side;
    let player_srgba = player.pawn_color.to_srgba();

    for (tile, material_handle) in &board_tiles {
        let shade_index = (tile.coord.q - tile.coord.r).rem_euclid(3);
        let color = if tile.coord.is_on_side(goal_side, turn_state.board_radius) {
            highlighted_goal_tile_color(player_srgba, shade_index)
        } else {
            normal_tile_color(shade_index)
        };
        if let Some(material) = materials.get_mut(material_handle) {
            material.color = color;
        }
    }
}

fn highlighted_goal_tile_color(player_color: Srgba, shade_index: i32) -> Color {
    let shade_multiplier = match shade_index {
        0 => 1.0,
        1 => 0.9,
        _ => 0.8,
    };

    let lift = 0.35;
    let red = (player_color.red + (1.0 - player_color.red) * lift) * shade_multiplier;
    let green = (player_color.green + (1.0 - player_color.green) * lift) * shade_multiplier;
    let blue = (player_color.blue + (1.0 - player_color.blue) * lift) * shade_multiplier;

    Color::srgb(red.min(1.0), green.min(1.0), blue.min(1.0))
}

fn normal_tile_color(shade_index: i32) -> Color {
    // A 3-coloring of axial coordinates. For every hex neighbor step,
    // `(q - r) mod 3` changes, so adjacent tiles always differ.
    match shade_index {
        0 => Color::srgb(0.25, 0.55, 0.35),
        1 => Color::srgb(0.21, 0.48, 0.31),
        _ => Color::srgb(0.17, 0.42, 0.27),
    }
}
