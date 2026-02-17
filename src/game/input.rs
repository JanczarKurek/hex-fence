use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::hex_grid::AxialCoord;

use super::audio::GameSoundEvent;
use super::components::Pawn;
use super::fence::{self, FencePlacementState};
use super::selection::PawnSelection;
use super::state::{ActionOutcome, TurnState};

pub fn move_current_pawn_on_click(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut turn_state: ResMut<TurnState>,
    mut selection: ResMut<PawnSelection>,
    mut fence_placement: ResMut<FencePlacementState>,
    mut pawn_query: Query<(&Pawn, &mut Transform)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut sound_events: EventWriter<GameSoundEvent>,
) {
    if keys.just_pressed(KeyCode::KeyF) {
        fence_placement.enabled = !fence_placement.enabled;
        selection.current_selected = false;
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        fence_placement.shape = fence_placement.shape.next();
    }
    if keys.just_pressed(KeyCode::KeyE) {
        fence_placement.orientation = (fence_placement.orientation + 1) % 6;
    }

    if !mouse_buttons.just_pressed(MouseButton::Left) || turn_state.winner.is_some() {
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

    let target = AxialCoord::from_world(world_pos);
    if !target.is_inside_board(turn_state.board_radius) {
        return;
    }
    sound_events.write(GameSoundEvent::Click);

    if fence_placement.enabled {
        let edges = fence::fence_edges(target, fence_placement.shape, fence_placement.orientation);
        let current = turn_state.current_player;
        if turn_state.try_place_fence(&edges).is_err() {
            return;
        }

        let color = turn_state.players[current].pawn_color;
        fence::spawn_fence_segments(&mut commands, &mut meshes, &mut materials, &edges, color);
        selection.current_selected = false;
        sound_events.write(GameSoundEvent::MovePawn);
        return;
    }

    let current = turn_state.current_player;
    let current_pos = turn_state.pawn_positions[current];

    if target == current_pos {
        selection.current_selected = !selection.current_selected;
        if selection.current_selected {
            sound_events.write(GameSoundEvent::SelectPawn);
        }
        return;
    }

    if !selection.current_selected {
        return;
    }

    let legal_moves = turn_state.legal_moves_for_current();
    if !legal_moves.contains(&target) {
        selection.current_selected = false;
        return;
    }

    if let Ok(outcome) = turn_state.try_move_current_pawn(target) {
        sound_events.write(GameSoundEvent::MovePawn);

        for (pawn, mut transform) in &mut pawn_query {
            if pawn.player_index == current {
                let world = target.to_world();
                transform.translation = Vec3::new(world.x, world.y, 2.0);
                break;
            }
        }

        if matches!(outcome, ActionOutcome::Won(_)) {
            sound_events.write(GameSoundEvent::Win);
        }
    }

    selection.current_selected = false;
}
