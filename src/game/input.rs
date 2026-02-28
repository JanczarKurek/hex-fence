use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::app_state::GameConfig;
use crate::camera::MainCamera;
use crate::hex_grid::AxialCoord;
use crate::network::{NetConfig, NetRuntime};
use crate::settings::AppSettings;

use super::actions::{ActionSource, GameActionRequest};
use super::audio::GameSoundEvent;
use super::fence::{self, FencePlacementState};
use super::selection::PawnSelection;
use super::state::{GameAction, TurnState};

pub fn move_current_pawn_on_click(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    turn_state: Res<TurnState>,
    game_config: Res<GameConfig>,
    app_settings: Res<AppSettings>,
    net_config: Res<NetConfig>,
    net_runtime: Res<NetRuntime>,
    mut selection: ResMut<PawnSelection>,
    mut fence_placement: ResMut<FencePlacementState>,
    mut action_requests: EventWriter<GameActionRequest>,
    mut sound_events: EventWriter<GameSoundEvent>,
) {
    let toggle_key = app_settings.controls.toggle_fence_mode_key();
    let cycle_shape_key = app_settings.controls.cycle_fence_shape_key();
    let rotate_key = app_settings.controls.rotate_fence_orientation_key();

    if keys.just_pressed(toggle_key) {
        fence_placement.enabled = !fence_placement.enabled;
        selection.current_selected = false;
    }
    if keys.just_pressed(cycle_shape_key) {
        fence_placement.shape = fence_placement.shape.next();
    }
    if keys.just_pressed(rotate_key) {
        fence_placement.orientation = (fence_placement.orientation + 1) % 6;
    }

    if !net_runtime.can_control_player(&net_config, turn_state.current_player) {
        return;
    }

    if game_config
        .player_control(turn_state.current_player)
        .is_ai()
    {
        return;
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
        if !turn_state.can_place_fence(&edges) {
            return;
        }

        action_requests.write(GameActionRequest {
            source: ActionSource::Local,
            action: GameAction::PlaceFence { edges },
        });
        selection.current_selected = false;
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

    action_requests.write(GameActionRequest {
        source: ActionSource::Local,
        action: GameAction::Move { target },
    });
    selection.current_selected = false;
}
