pub mod actions;
mod ai;
pub mod audio;
mod components;
mod fence;
mod highlight;
mod input;
mod player;
#[cfg(test)]
mod rules_tests;
mod selection;
mod spawn;
pub mod state;
mod ui;
mod utils;

use crate::app_state::{AppPhase, GameConfig, StartRematch};
use bevy::prelude::*;

use ai::{AiRng, AiTurnCooldown};
use components::{InGameHudUi, MoveHighlight, Pawn, PlayerPanelUiState};
use fence::FencePlacementState;
use selection::PawnSelection;
use state::TurnState;

pub(crate) use utils::despawn_all;

pub struct GamePlugin;
pub use components::HoveredGoalPreview;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TurnState::default())
            .insert_resource(PawnSelection::default())
            .insert_resource(FencePlacementState::default())
            .insert_resource(HoveredGoalPreview::default())
            .insert_resource(PlayerPanelUiState::default())
            .insert_resource(AiRng::default())
            .insert_resource(AiTurnCooldown::default())
            .init_resource::<audio::GameAudioAssets>()
            .add_event::<actions::GameActionRequest>()
            .add_event::<actions::GameActionApplied>()
            .add_event::<audio::GameSoundEvent>()
            .add_systems(Startup, audio::start_background_music)
            .add_systems(
                OnEnter(AppPhase::InGame),
                (
                    state::reset_turn_state_from_config,
                    selection::reset_selection,
                    fence::reset_fence_placement,
                    spawn::spawn_pawns,
                    ui::setup_turn_indicator,
                )
                    .chain(),
            )
            .add_systems(OnExit(AppPhase::InGame), cleanup_in_game_entities)
            .add_systems(
                Update,
                (
                    handle_start_rematch,
                    (
                        ai::random_ai_take_turn,
                        input::move_current_pawn_on_click,
                        actions::apply_game_action_requests,
                        fence::update_fence_preview,
                        audio::play_sound_effects,
                    )
                        .chain(),
                    audio::update_background_music_volume,
                    ui::handle_player_panel_toggle_button,
                    ui::sync_player_panel_collapsed_state,
                    highlight::update_move_highlights,
                    ui::update_hovered_goal_preview,
                    ui::update_turn_indicator,
                )
                    .run_if(in_state(AppPhase::InGame)),
            );
    }
}

fn _cleanup_board(
    commands: &mut Commands,
    pawns: Query<Entity, With<Pawn>>,
    fence_segments: Query<Entity, With<fence::FenceSegment>>,
    fence_previews: Query<Entity, With<fence::FencePreviewSegment>>,
    move_highlights: Query<Entity, With<MoveHighlight>>,
) {
    despawn_all!(commands, pawns);
    despawn_all!(commands, fence_segments);
    despawn_all!(commands, fence_previews);
    despawn_all!(commands, move_highlights);
}

fn cleanup_in_game_entities(
    mut commands: Commands,
    pawns: Query<Entity, With<Pawn>>,
    fence_segments: Query<Entity, With<fence::FenceSegment>>,
    fence_previews: Query<Entity, With<fence::FencePreviewSegment>>,
    move_highlights: Query<Entity, With<MoveHighlight>>,
    hud_entities: Query<Entity, With<InGameHudUi>>,
) {
    _cleanup_board(
        &mut commands,
        pawns,
        fence_segments,
        fence_previews,
        move_highlights,
    );
    for entity in &hud_entities {
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).despawn();
    }
}

fn handle_start_rematch(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    mut rematch_events: EventReader<StartRematch>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut turn_state: ResMut<TurnState>,
    mut selection: ResMut<PawnSelection>,
    mut fence_placement: ResMut<FencePlacementState>,
    mut ai_cooldown: ResMut<AiTurnCooldown>,
    pawns: Query<Entity, With<Pawn>>,
    fence_segments: Query<Entity, With<fence::FenceSegment>>,
    fence_previews: Query<Entity, With<fence::FencePreviewSegment>>,
    move_highlights: Query<Entity, With<MoveHighlight>>,
) {
    let mut restart_requested = false;
    for _ in rematch_events.read() {
        restart_requested = true;
    }
    if !restart_requested {
        return;
    }

    _cleanup_board(
        &mut commands,
        pawns,
        fence_segments,
        fence_previews,
        move_highlights,
    );

    *turn_state = TurnState::with_player_colors(
        game_config.player_count,
        game_config.board_radius,
        &game_config.player_colors,
    );
    selection.current_selected = false;
    *fence_placement = FencePlacementState::default();
    *ai_cooldown = AiTurnCooldown::default();
    spawn::spawn_pawn_entities(&mut commands, &mut meshes, &mut materials, &turn_state);
}
