pub mod audio;
mod components;
mod highlight;
mod input;
mod player;
mod selection;
mod spawn;
mod state;
mod ui;

use bevy::prelude::*;

use selection::PawnSelection;
use state::TurnState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TurnState::new_three_players())
            .insert_resource(PawnSelection::default())
            .init_resource::<audio::GameAudioAssets>()
            .add_event::<audio::GameSoundEvent>()
            .add_systems(
                Startup,
                (
                    audio::start_background_music,
                    spawn::spawn_pawns,
                    ui::setup_turn_indicator,
                ),
            )
            .add_systems(
                Update,
                (
                    input::move_current_pawn_on_click,
                    audio::play_sound_effects.after(input::move_current_pawn_on_click),
                    audio::update_background_music_volume,
                    highlight::update_move_highlights,
                    ui::update_turn_indicator,
                ),
            );
    }
}
