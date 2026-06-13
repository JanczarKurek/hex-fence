use bevy::math::primitives::Rectangle;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::hex_grid::{AxialCoord, HexRender, TILE_RADIUS};

use super::state::{EdgeKey, GameState};

use super::utils::despawn_all;

pub use giereczka_core::fence_rules::{FenceShape, fence_edges};

#[derive(Resource)]
pub struct FencePlacementState {
    pub enabled: bool,
    pub shape: FenceShape,
    pub orientation: usize,
}

impl Default for FencePlacementState {
    fn default() -> Self {
        Self {
            enabled: false,
            shape: FenceShape::C,
            orientation: 0,
        }
    }
}

pub fn reset_fence_placement(mut placement: ResMut<FencePlacementState>) {
    *placement = FencePlacementState::default();
}

#[derive(Component)]
pub struct FenceSegment;

#[derive(Component)]
pub struct FencePreviewSegment;

pub fn spawn_fence_segments(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    edges: &[EdgeKey],
    color: Color,
) {
    let mesh = meshes.add(Rectangle::new(TILE_RADIUS * 1.0, 7.0));
    let material = materials.add(color);

    for edge in edges {
        let a = edge.a.to_world();
        let b = edge.b.to_world();
        let midpoint = (a + b) * 0.5;
        let delta = b - a;
        let angle = delta.y.atan2(delta.x) + core::f32::consts::FRAC_PI_2;

        commands.spawn((
            FenceSegment,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_translation(midpoint.extend(1.5))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

pub fn update_fence_preview(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    turn_state: Res<GameState>,
    fence_placement: Res<FencePlacementState>,
    existing_preview: Query<Entity, With<FencePreviewSegment>>,
) {
    despawn_all!(commands, existing_preview);

    if !fence_placement.enabled || turn_state.winner.is_some() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let anchor = AxialCoord::from_world(world_pos);
    if !anchor.is_inside_board(turn_state.board_radius) {
        return;
    }

    let edges = fence_edges(anchor, fence_placement.shape, fence_placement.orientation);
    let legal = turn_state.can_place_fence(&edges);
    let color = if legal {
        Color::srgba(0.4, 0.95, 0.55, 0.45)
    } else {
        Color::srgba(1.0, 0.3, 0.3, 0.45)
    };

    let mesh = meshes.add(Rectangle::new(TILE_RADIUS * 1.0, 7.0));
    let material = materials.add(color);

    for edge in edges {
        let a = edge.a.to_world();
        let b = edge.b.to_world();
        let midpoint = (a + b) * 0.5;
        let delta = b - a;
        let angle = delta.y.atan2(delta.x) + core::f32::consts::FRAC_PI_2;

        commands.spawn((
            FencePreviewSegment,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_translation(midpoint.extend(1.6))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}
