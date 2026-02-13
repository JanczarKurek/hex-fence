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
            .add_systems(Startup, (spawn::spawn_pawns, ui::setup_turn_indicator))
            .add_systems(
                Update,
                (
                    input::move_current_pawn_on_click,
                    highlight::update_move_highlights,
                    ui::update_turn_indicator,
                ),
            );
    }
}
