//! In-game neural AI backend (`AiStrategy::Neural`).
//!
//! Loads `models/current.onnx` (override with `GIERECZKA_MODEL_PATH`) when a game starts with a
//! Neural player, guarding on the model's radius sidecar. Inference runs MCTS guided by the network
//! (the same search the `eval`/`selfplay` binaries use) — that search is where the strength comes
//! from: the trained net beats the heuristic ~0.72 with MCTS but only ~parity on the raw policy
//! alone. Sim count is `GIERECZKA_MCTS_SIMS` (default 64; 64-128 is the sweet spot). Any failure —
//! missing model, radius mismatch, or ONNX Runtime not available (`ORT_DYLIB_PATH` unset) — leaves
//! the backend empty and the AI falls back to the heuristic.

use bevy::prelude::*;

use giereczka_core::encoding::Encoder;
use giereczka_core::heuristic::AiRng;
use giereczka_core::mcts::{MctsConfig, run_mcts};
use giereczka_core::onnx::OnnxEvaluator;
use giereczka_core::state::{GameAction, TurnState};

use crate::app_state::{AiStrategy, GameConfig};

/// Default MCTS simulations per move when `GIERECZKA_MCTS_SIMS` is unset. 64 matches the strongest
/// measured play of the trained champion while keeping in-game turns snappy.
const DEFAULT_MCTS_SIMS: usize = 64;

struct NeuralBackend {
    evaluator: OnnxEvaluator,
    encoder: Encoder,
    sims: usize,
}

#[derive(Resource, Default)]
pub struct NeuralAi {
    backend: Option<NeuralBackend>,
}

impl NeuralAi {
    /// Best action per a network-guided MCTS search (temperature 0, no exploration noise), or
    /// `None` if no model is loaded. Deterministic-ish strong play, matching the eval harness.
    pub fn choose_action(&self, state: &TurnState, rng: &mut AiRng) -> Option<GameAction> {
        let backend = self.backend.as_ref()?;
        let config = MctsConfig {
            simulations: backend.sims,
            temperature: 0.0,
            dirichlet_epsilon: 0.0,
            ..MctsConfig::default()
        };
        run_mcts(state, &backend.encoder, &backend.evaluator, config, rng).map(|result| result.action)
    }
}

fn model_path() -> String {
    std::env::var("GIERECZKA_MODEL_PATH").unwrap_or_else(|_| "models/current.onnx".to_string())
}

/// MCTS simulations per move, from `GIERECZKA_MCTS_SIMS` (falls back to [`DEFAULT_MCTS_SIMS`]).
fn mcts_sims() -> usize {
    std::env::var("GIERECZKA_MCTS_SIMS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&sims| sims > 0)
        .unwrap_or(DEFAULT_MCTS_SIMS)
}

/// Trained board radius from the `<model>.json` sidecar written by `export_onnx.py`.
fn model_radius(path: &str) -> Option<i32> {
    let text = std::fs::read_to_string(format!("{path}.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    value.get("radius")?.as_i64().map(|r| r as i32)
}

pub fn setup_neural_ai(mut neural: ResMut<NeuralAi>, game_config: Res<GameConfig>) {
    neural.backend = None;

    let uses_neural = (0..game_config.player_count).any(|player| {
        game_config.player_control(player).is_ai()
            && game_config.player_ai_strategy(player) == AiStrategy::Neural
    });
    if !uses_neural {
        return;
    }

    let path = model_path();
    let radius = game_config.board_radius;
    match model_radius(&path) {
        Some(model_radius) if model_radius != radius => {
            warn!(
                "Neural AI: model {path} radius {model_radius} != board radius {radius}; \
                 falling back to Heuristic"
            );
            return;
        }
        None => {
            warn!("Neural AI: {path}.json missing/unreadable; falling back to Heuristic");
            return;
        }
        _ => {}
    }

    let encoder = Encoder::new(radius);
    match OnnxEvaluator::from_file(&path, encoder.dim()) {
        Ok(evaluator) => {
            let sims = mcts_sims();
            info!("Neural AI: loaded {path} (radius {radius}, MCTS {sims} sims)");
            neural.backend = Some(NeuralBackend {
                evaluator,
                encoder,
                sims,
            });
        }
        Err(error) => {
            warn!(
                "Neural AI: failed to load {path} ({error}); falling back to Heuristic \
                 (is ORT_DYLIB_PATH set?)"
            );
        }
    }
}
