//! In-game neural AI backend (`AiStrategy::Neural`).
//!
//! Loads `models/current.onnx` (override with `GIERECZKA_MODEL_PATH`) when a game starts with a
//! Neural player, guarding on the model's radius sidecar. Inference uses a single policy forward
//! pass (`policy_best_action`) for snappy turns. Any failure — missing model, radius mismatch, or
//! ONNX Runtime not available (`ORT_DYLIB_PATH` unset) — leaves the backend empty and the AI
//! falls back to the heuristic.

use bevy::prelude::*;

use giereczka_core::encoding::Encoder;
use giereczka_core::mcts::policy_best_action;
use giereczka_core::onnx::OnnxEvaluator;
use giereczka_core::state::{GameAction, TurnState};

use crate::app_state::{AiStrategy, GameConfig};

struct NeuralBackend {
    evaluator: OnnxEvaluator,
    encoder: Encoder,
}

#[derive(Resource, Default)]
pub struct NeuralAi {
    backend: Option<NeuralBackend>,
}

impl NeuralAi {
    /// Best action per the network's policy, or `None` if no model is loaded.
    pub fn choose_action(&self, state: &TurnState) -> Option<GameAction> {
        let backend = self.backend.as_ref()?;
        policy_best_action(state, &backend.encoder, &backend.evaluator)
    }
}

fn model_path() -> String {
    std::env::var("GIERECZKA_MODEL_PATH").unwrap_or_else(|_| "models/current.onnx".to_string())
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
            info!("Neural AI: loaded {path} (radius {radius})");
            neural.backend = Some(NeuralBackend { evaluator, encoder });
        }
        Err(error) => {
            warn!(
                "Neural AI: failed to load {path} ({error}); falling back to Heuristic \
                 (is ORT_DYLIB_PATH set?)"
            );
        }
    }
}
