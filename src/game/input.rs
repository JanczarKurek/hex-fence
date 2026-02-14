use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::hex_grid::AxialCoord;

use super::audio::GameSoundEvent;
use super::components::Pawn;
use super::selection::PawnSelection;
use super::state::TurnState;

pub fn move_current_pawn_on_click(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut turn_state: ResMut<TurnState>,
    mut selection: ResMut<PawnSelection>,
    mut pawn_query: Query<(&Pawn, &mut Transform)>,
    mut sound_events: EventWriter<GameSoundEvent>,
) {
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
    if !target.is_inside_board() {
        return;
    }
    sound_events.write(GameSoundEvent::Click);

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

    turn_state.pawn_positions[current] = target;
    sound_events.write(GameSoundEvent::MovePawn);

    for (pawn, mut transform) in &mut pawn_query {
        if pawn.player_index == current {
            let world = target.to_world();
            transform.translation = Vec3::new(world.x, world.y, 2.0);
            break;
        }
    }

    if target.is_on_side(turn_state.players[current].goal_side) {
        turn_state.winner = Some(current);
        sound_events.write(GameSoundEvent::Win);
        selection.current_selected = false;
        return;
    }

    turn_state.advance_turn();
    selection.current_selected = false;
}
