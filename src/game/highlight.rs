use bevy::math::primitives::RegularPolygon;
use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;

use crate::hex_grid::TILE_RADIUS;

use super::components::MoveHighlight;
use super::selection::PawnSelection;
use super::state::TurnState;

pub fn update_move_highlights(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    turn_state: Res<TurnState>,
    selection: Res<PawnSelection>,
    existing: Query<Entity, With<MoveHighlight>>,
) {
    if !turn_state.is_changed() && !selection.is_changed() {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    if !selection.current_selected || turn_state.winner.is_some() {
        return;
    }

    let mesh = meshes.add(RegularPolygon::new(TILE_RADIUS * 0.92, 6));
    let material = materials.add(Color::srgba(1.0, 1.0, 1.0, 0.22));

    for destination in turn_state.legal_moves_for_current() {
        let world = destination.to_world();
        commands.spawn((
            MoveHighlight,
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
            Transform::from_xyz(world.x, world.y, 1.0),
        ));
    }
}
