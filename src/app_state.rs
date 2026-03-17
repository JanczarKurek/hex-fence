use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const MAX_PLAYERS: usize = 6;
pub const DEFAULT_PLAYER_CONTROLS: [PlayerControl; MAX_PLAYERS] =
    [PlayerControl::Human; MAX_PLAYERS];
pub const DEFAULT_AI_STRATEGIES: [AiStrategy; MAX_PLAYERS] = [AiStrategy::Heuristic; MAX_PLAYERS];
pub const DEFAULT_PLAYER_COLORS: [PlayerColor; MAX_PLAYERS] = [
    PlayerColor::Red,
    PlayerColor::Blue,
    PlayerColor::Gold,
    PlayerColor::Teal,
    PlayerColor::Pink,
    PlayerColor::Orange,
];

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
    pub player_ai_strategies: [AiStrategy; 6],
    pub player_colors: [PlayerColor; 6],
    pub ai_cooldown_seconds: f32,
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

impl AiStrategy {
    pub fn short_label(self) -> &'static str {
        match self {
            Self::Heuristic => " [AI:H]",
            Self::AlphaBeta => " [AI:AB]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerColor {
    Red,
    Blue,
    Gold,
    Teal,
    Pink,
    Orange,
}

impl PlayerColor {
    pub const ALL: [Self; MAX_PLAYERS] = DEFAULT_PLAYER_COLORS;

    pub fn color(self) -> Color {
        match self {
            Self::Red => Color::srgb(0.92, 0.28, 0.24),
            Self::Blue => Color::srgb(0.22, 0.56, 0.92),
            Self::Gold => Color::srgb(0.95, 0.75, 0.2),
            Self::Teal => Color::srgb(0.22, 0.82, 0.65),
            Self::Pink => Color::srgb(0.96, 0.45, 0.86),
            Self::Orange => Color::srgb(0.98, 0.56, 0.22),
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Red => "R",
            Self::Blue => "B",
            Self::Gold => "G",
            Self::Teal => "T",
            Self::Pink => "P",
            Self::Orange => "O",
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board_radius: 4,
            player_count: 3,
            player_controls: DEFAULT_PLAYER_CONTROLS,
            player_ai_strategies: DEFAULT_AI_STRATEGIES,
            player_colors: DEFAULT_PLAYER_COLORS,
            ai_cooldown_seconds: 1.0,
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

    pub fn player_ai_strategy(&self, player_index: usize) -> AiStrategy {
        self.player_ai_strategies
            .get(player_index)
            .copied()
            .unwrap_or(AiStrategy::Heuristic)
    }
}
