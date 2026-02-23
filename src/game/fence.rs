use bevy::math::primitives::Rectangle;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::hex_grid::{AxialCoord, TILE_RADIUS};

use super::state::EdgeKey;
use super::state::TurnState;

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FenceShape {
    S,
    SMirrored,
    C,
    Y,
}

impl FenceShape {
    pub fn next(self) -> Self {
        match self {
            Self::S => Self::SMirrored,
            Self::SMirrored => Self::C,
            Self::C => Self::Y,
            Self::Y => Self::S,
        }
    }
}

#[derive(Component)]
pub struct FenceSegment;

#[derive(Component)]
pub struct FencePreviewSegment;

pub fn fence_edges(anchor: AxialCoord, shape: FenceShape, orientation: usize) -> [EdgeKey; 3] {
    let o = orientation % 6;
    let n0 = anchor.neighbor_in_direction(o);
    let n1 = anchor.neighbor_in_direction((o + 1) % 6);

    match shape {
        FenceShape::C => [
            EdgeKey::from_cells(anchor, n0),
            EdgeKey::from_cells(anchor, n1),
            EdgeKey::from_cells(anchor, anchor.neighbor_in_direction((o + 2) % 6)),
        ],
        // Three fence segments sharing one common hex-grid vertex.
        FenceShape::Y => [
            EdgeKey::from_cells(anchor, n0),
            EdgeKey::from_cells(anchor, n1),
            EdgeKey::from_cells(n0, n1),
        ],
        // Connected zig-zag path of three segments.
        FenceShape::S => {
            // Chain: (anchor-n0) -> (anchor-n1) -> (n1-next)
            // where each neighboring pair shares a fence endpoint.
            let next = n1.neighbor_in_direction((o + 3) % 6);
            [
                EdgeKey::from_cells(anchor, n0),
                EdgeKey::from_cells(anchor, n1),
                EdgeKey::from_cells(n1, next),
            ]
        }
        // Mirrored connected zig-zag path of three segments.
        FenceShape::SMirrored => {
            // Chain: (anchor-n0) -> (anchor-n1) -> (n0-next)
            // where each neighboring pair shares a fence endpoint.
            let next = n0.neighbor_in_direction((o + 4) % 6);
            [
                EdgeKey::from_cells(anchor, n0),
                EdgeKey::from_cells(anchor, n1),
                EdgeKey::from_cells(n0, next),
            ]
        }
    }
}

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
    turn_state: Res<TurnState>,
    fence_placement: Res<FencePlacementState>,
    existing_preview: Query<Entity, With<FencePreviewSegment>>,
) {
    for entity in &existing_preview {
        commands.entity(entity).despawn();
    }

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
