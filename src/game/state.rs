use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

use crate::app_state::GameConfig;

pub use giereczka_core::state::{ActionOutcome, AppliedAction, EdgeKey, GameAction, TurnState};

/// Bevy `Resource` wrapper around the Bevy-free [`TurnState`] rules engine.
///
/// `TurnState` lives in `giereczka-core` and cannot derive `Resource` (orphan rule),
/// so the game wraps it. `Deref`/`DerefMut` keep every call site (`turn_state.players`,
/// `turn_state.try_apply_action(..)`) unchanged; only system parameter types switch
/// from `Res<TurnState>` to `Res<GameState>`.
#[derive(Resource, Default)]
pub struct GameState(pub TurnState);

impl Deref for GameState {
    type Target = TurnState;

    fn deref(&self) -> &TurnState {
        &self.0
    }
}

impl DerefMut for GameState {
    fn deref_mut(&mut self) -> &mut TurnState {
        &mut self.0
    }
}

pub fn reset_turn_state_from_config(
    game_config: Res<GameConfig>,
    mut turn_state: ResMut<GameState>,
) {
    turn_state.0 = TurnState::new(game_config.player_count, game_config.board_radius);
}
