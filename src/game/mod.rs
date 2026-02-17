pub mod audio;
mod components;
mod fence;
mod highlight;
mod input;
mod player;
mod selection;
mod spawn;
mod state;
mod ui;
#[cfg(test)]
mod rules_tests;

use bevy::prelude::*;
use crate::app_state::AppPhase;

use fence::FencePlacementState;
use selection::PawnSelection;
use state::TurnState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TurnState::default())
            .insert_resource(PawnSelection::default())
            .insert_resource(FencePlacementState::default())
            .init_resource::<audio::GameAudioAssets>()
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
                    input::move_current_pawn_on_click,
                    fence::update_fence_preview.after(input::move_current_pawn_on_click),
                    audio::play_sound_effects.after(input::move_current_pawn_on_click),
                    audio::update_background_music_volume,
                    highlight::update_move_highlights,
                    ui::update_turn_indicator,
                )
                    .run_if(in_state(AppPhase::InGame)),
            );
    }
}
