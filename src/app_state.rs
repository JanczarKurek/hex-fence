use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AppPhase {
    #[default]
    Menu,
    InGame,
}

#[derive(Event, Debug, Clone, Copy, Default)]
pub struct RematchRequested;

#[derive(Event, Debug, Clone, Copy, Default)]
pub struct StartRematch;

#[derive(Resource, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameConfig {
    pub board_radius: i32,
    pub player_count: usize,
    pub player_controls: [PlayerControl; 6],
    pub ai_cooldown_seconds: f32,
    pub ai_strategy: AiStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerControl {
    Human,
    RandomAi,
}

impl PlayerControl {
    pub fn is_ai(self) -> bool {
        matches!(self, Self::RandomAi)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiStrategy {
    Heuristic,
    AlphaBeta,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board_radius: 4,
            player_count: 3,
            player_controls: [
                PlayerControl::Human,
                PlayerControl::Human,
                PlayerControl::Human,
                PlayerControl::Human,
                PlayerControl::Human,
                PlayerControl::Human,
            ],
            ai_cooldown_seconds: 1.0,
            ai_strategy: AiStrategy::Heuristic,
        }
    }
}

impl GameConfig {
    pub fn player_control(&self, player_index: usize) -> PlayerControl {
        self.player_controls
            .get(player_index)
            .copied()
            .unwrap_or(PlayerControl::Human)
    }
}
