use bevy::prelude::*;

use crate::app_state::{AiStrategy, PlayerColor, PlayerControl};
use crate::network::{NetLobbyState, NetMode};
use crate::settings::AppSettings;

#[derive(Resource)]
pub(super) struct MenuSelection {
    pub(super) screen: MenuScreen,
    pub(super) game_mode: StartGameMode,
    pub(super) board_radius: i32,
    pub(super) player_count: usize,
    pub(super) player_controls: [PlayerControl; 6],
    pub(super) player_ai_strategies: [AiStrategy; 6],
    pub(super) player_colors: [PlayerColor; 6],
    pub(super) ai_cooldown_ms: u32,
    pub(super) net_mode: NetMode,
    pub(super) net_address: String,
    pub(super) address_focused: bool,
    pub(super) show_authors_popup: bool,
    pub(super) show_rules_popup: bool,
    pub(super) show_settings_popup: bool,
    pub(super) network_local_slot: Option<usize>,
    pub(super) open_player_detail_dropdown: Option<usize>,
}

impl Default for MenuSelection {
    fn default() -> Self {
        Self {
            screen: MenuScreen::ModeSelect,
            game_mode: StartGameMode::Local,
            board_radius: 4,
            player_count: 3,
            player_controls: [PlayerControl::Human; 6],
            player_ai_strategies: [AiStrategy::Heuristic; 6],
            player_colors: [
                PlayerColor::Red,
                PlayerColor::Blue,
                PlayerColor::Gold,
                PlayerColor::Teal,
                PlayerColor::Pink,
                PlayerColor::Orange,
            ],
            ai_cooldown_ms: 1_000,
            net_mode: NetMode::Local,
            net_address: "127.0.0.1:4000".to_string(),
            address_focused: false,
            show_authors_popup: false,
            show_rules_popup: false,
            show_settings_popup: false,
            network_local_slot: Some(0),
            open_player_detail_dropdown: None,
        }
    }
}

#[derive(Component)]
pub(super) struct StartMenuRoot;

#[derive(Component)]
pub(super) struct MenuMainPanel;

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
pub(super) struct PlayerControlButton {
    pub(super) player_index: usize,
}

#[derive(Component)]
pub(super) struct PlayerControlToggleButton {
    pub(super) player_index: usize,
}

#[derive(Component)]
pub(super) struct AiCooldownButton {
    pub(super) cooldown_ms: u32,
}

#[derive(Component)]
pub(super) struct PlayerDetailDropdownButton {
    pub(super) player_index: usize,
}

#[derive(Component)]
pub(super) struct PlayerDetailDropdownText {
    pub(super) player_index: usize,
}

#[derive(Component)]
pub(super) struct PlayerDetailDropdownMenu {
    pub(super) player_index: usize,
}

#[derive(Component)]
pub(super) struct PlayerDetailOptionButton {
    pub(super) player_index: usize,
    pub(super) option: PlayerDetailOption,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlayerDetailOption {
    Host,
    Client,
    Heuristic,
    AlphaBeta,
}

#[derive(Component)]
pub(super) struct PlayerColorButton {
    pub(super) player_index: usize,
    pub(super) color: PlayerColor,
}

#[derive(Component)]
pub(super) struct PlayerSetupRow {
    pub(super) player_index: usize,
}

#[derive(Component)]
pub(super) struct PlayerAiOnly {
    pub(super) player_index: usize,
}

#[derive(Component)]
pub(super) struct LobbyPlayerListScroll;

#[derive(Component)]
pub(super) struct MainMenuActionButton {
    pub(super) action: MainMenuAction,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum MainMenuAction {
    LocalGame,
    NetworkGame,
    Settings,
    Rules,
    Authors,
    Quit,
}

#[derive(Component)]
pub(super) struct BackToModeButton;

#[derive(Component)]
pub(super) struct MenuScreenModeSelect;

#[derive(Component)]
pub(super) struct MenuScreenSetup;

#[derive(Component)]
pub(super) struct MenuScreenNetworkLobby;

#[derive(Component)]
pub(super) struct AuthorsPopup;

#[derive(Component)]
pub(super) struct RulesPopup;

#[derive(Component)]
pub(super) struct MenuSettingsPopup;

#[derive(Component)]
pub(super) struct MenuSettingsCloseButton;

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
pub(super) struct NetworkAddressInputField;

#[derive(Component)]
pub(super) struct NetworkConnectButton;

#[derive(Component)]
pub(super) struct ConnectedPlayersText;

#[derive(Component)]
pub(super) struct NetworkSlotButton {
    pub(super) slot: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NetworkSlotOwner {
    Ai,
    Host,
    Client,
}

#[derive(Component)]
pub(super) struct NetworkLobbyHostOnly;

#[derive(Component)]
pub(super) struct NetworkLobbyClientOnly;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum MenuScreen {
    ModeSelect,
    Setup,
    NetworkLobby,
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
    pub(super) pending_control_binding: Option<ControlBindingKind>,
}

impl Default for SettingsUiState {
    fn default() -> Self {
        Self {
            open: false,
            active_tab: SettingsTab::Sound,
            pending_control_binding: None,
        }
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub(super) enum SettingsTab {
    Sound,
    Controls,
}

#[derive(Component, Clone, Copy)]
pub(super) enum SoundSliderKind {
    Master,
    Music,
    Effects,
}

#[derive(Component, Clone, Copy, Eq, PartialEq)]
pub(super) enum ControlBindingKind {
    ToggleFenceMode,
    CycleFenceShape,
    RotateFenceOrientation,
}

#[derive(Component)]
pub(super) struct ControlBindingButton {
    pub(super) kind: ControlBindingKind,
}

#[derive(Component)]
pub(super) struct ControlBindingValueText {
    pub(super) kind: ControlBindingKind,
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

impl MenuSelection {
    pub(super) fn sync_from_network_lobby(
        &mut self,
        net_mode: NetMode,
        local_player_index: usize,
        net_lobby: &NetLobbyState,
    ) {
        self.board_radius = net_lobby.config.board_radius;
        self.player_count = net_lobby.config.player_count;
        self.player_controls = net_lobby.config.player_controls;
        self.player_ai_strategies = net_lobby.config.player_ai_strategies;
        self.player_colors = net_lobby.config.player_colors;
        self.ai_cooldown_ms = (net_lobby.config.ai_cooldown_seconds * 1000.0).round() as u32;
        self.network_local_slot = if matches!(net_mode, NetMode::Host) {
            net_lobby.host_slot
        } else {
            Some(local_player_index)
                .filter(|slot| *slot < net_lobby.config.player_count)
                .filter(|slot| net_lobby.remote_slots.contains(slot))
        };
    }
}
