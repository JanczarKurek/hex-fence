use bevy::prelude::*;
use std::time::Duration;

use crate::app_state::{AiStrategy, GameConfig};
use crate::network::{NetConfig, NetRuntime};

use giereczka_core::heuristic::{AiRng, choose_alpha_beta_action, choose_heuristic_action};

use super::actions::{ActionSource, GameActionRequest};
use super::neural::NeuralAi;
use super::state::GameState;

/// Bevy `Resource` wrapper around the Bevy-free [`AiRng`] from `giereczka-core`.
#[derive(Resource, Default)]
pub struct GameRng(pub AiRng);

#[derive(Resource)]
pub struct AiTurnCooldown {
    pending_player: Option<usize>,
    timer: Timer,
}

impl Default for AiTurnCooldown {
    fn default() -> Self {
        Self {
            pending_player: None,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        }
    }
}

impl AiTurnCooldown {
    fn clear(&mut self) {
        self.pending_player = None;
        self.timer.reset();
    }

    fn start_for_player(&mut self, player: usize, cooldown_seconds: f32) {
        self.pending_player = Some(player);
        self.timer
            .set_duration(Duration::from_secs_f32(cooldown_seconds.max(0.0)));
        self.timer.reset();
    }
}

pub fn random_ai_take_turn(
    time: Res<Time>,
    game_config: Res<GameConfig>,
    turn_state: Res<GameState>,
    net_config: Res<NetConfig>,
    net_runtime: Res<NetRuntime>,
    mut ai_rng: ResMut<GameRng>,
    mut ai_cooldown: ResMut<AiTurnCooldown>,
    neural_ai: Res<NeuralAi>,
    mut action_requests: EventWriter<GameActionRequest>,
) {
    if turn_state.winner.is_some() {
        ai_cooldown.clear();
        return;
    }

    let current_player = turn_state.current_player;
    if !game_config.player_control(current_player).is_ai() {
        ai_cooldown.clear();
        return;
    }

    if !net_runtime.can_control_player(&net_config, current_player) {
        ai_cooldown.clear();
        return;
    }

    if ai_cooldown.pending_player != Some(current_player) {
        ai_cooldown.start_for_player(current_player, game_config.ai_cooldown_seconds);
        return;
    }

    ai_cooldown.timer.tick(time.delta());
    if !ai_cooldown.timer.finished() {
        return;
    }

    let action = match game_config.player_ai_strategy(current_player) {
        AiStrategy::Heuristic => choose_heuristic_action(&turn_state.0, &mut ai_rng.0),
        AiStrategy::AlphaBeta => choose_alpha_beta_action(&turn_state.0, &mut ai_rng.0, 3)
            .or_else(|| choose_heuristic_action(&turn_state.0, &mut ai_rng.0)),
        AiStrategy::Neural => {
            // Network-guided MCTS (strong); falls back to the heuristic if no model is loaded.
            let mcts_action = neural_ai.choose_action(&turn_state.0, &mut ai_rng.0);
            mcts_action.or_else(|| choose_heuristic_action(&turn_state.0, &mut ai_rng.0))
        }
    };

    let Some(action) = action else {
        ai_cooldown.clear();
        return;
    };

    ai_cooldown.clear();
    action_requests.write(GameActionRequest {
        source: ActionSource::Local,
        action,
    });
}
