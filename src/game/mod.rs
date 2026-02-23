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

use crate::app_state::AppPhase;
use bevy::prelude::*;

use ai::{AiRng, AiTurnCooldown};
use fence::FencePlacementState;
use selection::PawnSelection;
use state::TurnState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TurnState::default())
            .insert_resource(PawnSelection::default())
            .insert_resource(FencePlacementState::default())
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
            .add_systems(
                Update,
                (
                    (
                        ai::random_ai_take_turn,
                        input::move_current_pawn_on_click,
                        actions::apply_game_action_requests,
                        fence::update_fence_preview,
                        audio::play_sound_effects,
                    )
                        .chain(),
                    audio::update_background_music_volume,
                    highlight::update_move_highlights,
                    ui::update_turn_indicator,
                )
                    .run_if(in_state(AppPhase::InGame)),
            );
    }
}
