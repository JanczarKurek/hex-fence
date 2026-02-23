use bevy::prelude::*;

use crate::app_state::AiStrategy;
use crate::network::NetMode;
use crate::settings::AppSettings;

#[derive(Resource)]
pub(super) struct MenuSelection {
    pub(super) screen: MenuScreen,
    pub(super) game_mode: StartGameMode,
    pub(super) board_radius: i32,
    pub(super) player_count: usize,
    pub(super) ai_player_count: usize,
    pub(super) ai_cooldown_ms: u32,
    pub(super) ai_strategy: AiStrategy,
    pub(super) net_mode: NetMode,
    pub(super) net_address: String,
    pub(super) address_focused: bool,
}

impl Default for MenuSelection {
    fn default() -> Self {
        Self {
            screen: MenuScreen::ModeSelect,
            game_mode: StartGameMode::Local,
            board_radius: 4,
            player_count: 3,
            ai_player_count: 0,
            ai_cooldown_ms: 1_000,
            ai_strategy: AiStrategy::Heuristic,
            net_mode: NetMode::Local,
            net_address: "127.0.0.1:4000".to_string(),
            address_focused: false,
        }
    }
}

#[derive(Component)]
pub(super) struct StartMenuRoot;

#[derive(Component)]
pub(super) struct StartGameButton;

#[derive(Component)]
pub(super) struct StartGameButtonLabel;

#[derive(Component)]
pub(super) struct BoardSizeButton {
    pub(super) radius: i32,
}

#[derive(Component)]
pub(super) struct PlayerCountButton {
    pub(super) player_count: usize,
}

#[derive(Component)]
pub(super) struct AiPlayerCountButton {
    pub(super) ai_player_count: usize,
}

#[derive(Component)]
pub(super) struct AiCooldownButton {
    pub(super) cooldown_ms: u32,
}

#[derive(Component)]
pub(super) struct AiStrategyButton {
    pub(super) strategy: AiStrategy,
}

#[derive(Component)]
pub(super) struct ModeChoiceButton {
    pub(super) mode: StartGameMode,
}

#[derive(Component)]
pub(super) struct BackToModeButton;

#[derive(Component)]
pub(super) struct MenuScreenModeSelect;

#[derive(Component)]
pub(super) struct MenuScreenSetup;

#[derive(Component)]
pub(super) struct LocalOnly;

#[derive(Component)]
pub(super) struct NetworkOnly;

#[derive(Component)]
pub(super) struct NetworkModeButton {
    pub(super) mode: NetMode,
}

#[derive(Component)]
pub(super) struct NetworkAddressInputButton;

#[derive(Component)]
pub(super) struct NetworkAddressText;

#[derive(Component)]
pub(super) struct NetworkConnectButton;

#[derive(Component)]
pub(super) struct ConnectedPlayersText;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum MenuScreen {
    ModeSelect,
    Setup,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum StartGameMode {
    Local,
    Network,
}

#[derive(Component)]
pub(super) struct ExitButton;

#[derive(Component)]
pub(super) struct InGameUiRoot;

#[derive(Component)]
pub(super) struct RematchButton;

#[derive(Component)]
pub(super) struct RematchPanel;

#[derive(Component)]
pub(super) struct SettingsToggleButton;

#[derive(Component)]
pub(super) struct SettingsPopup;

#[derive(Component)]
pub(super) struct SettingsTabButton {
    pub(super) tab: SettingsTab,
}

#[derive(Component)]
pub(super) struct SettingsTabContent {
    pub(super) tab: SettingsTab,
}

#[derive(Component)]
pub(super) struct SoundSliderTrack {
    pub(super) kind: SoundSliderKind,
}

#[derive(Component)]
pub(super) struct SoundSliderFill {
    pub(super) kind: SoundSliderKind,
}

#[derive(Component)]
pub(super) struct SoundSliderValueText {
    pub(super) kind: SoundSliderKind,
}

#[derive(Resource)]
pub(super) struct SettingsUiState {
    pub(super) open: bool,
    pub(super) active_tab: SettingsTab,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            open: false,
            active_tab: SettingsTab::Sound,
        }
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub(super) enum SettingsTab {
    Sound,
}

#[derive(Component, Clone, Copy)]
pub(super) enum SoundSliderKind {
    Master,
    Music,
    Effects,
}

impl SoundSliderKind {
    pub(super) fn value(self, settings: &AppSettings) -> f32 {
        match self {
            Self::Master => settings.audio.master,
            Self::Music => settings.audio.music,
            Self::Effects => settings.audio.effects,
        }
    }

    pub(super) fn set_value(self, settings: &mut AppSettings, value: f32) {
        let value = value.clamp(0.0, 1.0);
        match self {
            Self::Master => settings.audio.master = value,
            Self::Music => settings.audio.music = value,
            Self::Effects => settings.audio.effects = value,
        }
    }
}
