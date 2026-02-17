use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Default, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AppPhase {
    #[default]
    Menu,
    InGame,
}

#[derive(Resource, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameConfig {
    pub board_radius: i32,
    pub player_count: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            board_radius: 4,
            player_count: 3,
        }
    }
}
