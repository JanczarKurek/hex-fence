use bevy::prelude::*;

use crate::app_state::{
    AiStrategy, DEFAULT_AI_STRATEGIES, DEFAULT_PLAYER_COLORS, DEFAULT_PLAYER_CONTROLS, GameConfig,
    PlayerColor, PlayerControl,
};
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
            player_controls: DEFAULT_PLAYER_CONTROLS,
            player_ai_strategies: DEFAULT_AI_STRATEGIES,
            player_colors: DEFAULT_PLAYER_COLORS,
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

impl ControlBindingKind {
    pub(super) fn apply(self, app_settings: &mut AppSettings, key: KeyCode) -> bool {
        match self {
            Self::ToggleFenceMode => app_settings.controls.set_toggle_fence_mode_key(key),
            Self::CycleFenceShape => app_settings.controls.set_cycle_fence_shape_key(key),
            Self::RotateFenceOrientation => {
                app_settings.controls.set_rotate_fence_orientation_key(key)
            }
        }
    }

    pub(super) fn label(self, app_settings: &AppSettings) -> &'static str {
        match self {
            Self::ToggleFenceMode => app_settings.controls.toggle_fence_mode_label(),
            Self::CycleFenceShape => app_settings.controls.cycle_fence_shape_label(),
            Self::RotateFenceOrientation => app_settings.controls.rotate_fence_orientation_label(),
        }
    }
}

impl MenuSelection {
    fn display(active: bool) -> Display {
        if active { Display::Flex } else { Display::None }
    }

    pub(super) fn clear_overlays(&mut self) {
        self.show_authors_popup = false;
        self.show_rules_popup = false;
        self.show_settings_popup = false;
        self.open_player_detail_dropdown = None;
    }

    pub(super) fn uses_setup_layout(&self) -> bool {
        self.screen == MenuScreen::Setup
            && matches!(
                self.game_mode,
                StartGameMode::Local | StartGameMode::Network
            )
    }

    pub(super) fn main_panel_width(&self) -> (Val, Val) {
        if self.uses_setup_layout() {
            (Val::Px(980.0), Val::Percent(96.0))
        } else {
            (Val::Px(480.0), Val::Percent(92.0))
        }
    }

    pub(super) fn mode_select_display(&self) -> Display {
        Self::display(
            self.screen == MenuScreen::ModeSelect
                && !self.show_authors_popup
                && !self.show_rules_popup
                && !self.show_settings_popup,
        )
    }

    pub(super) fn setup_display(&self) -> Display {
        Self::display(self.screen == MenuScreen::Setup)
    }

    pub(super) fn network_lobby_display(&self) -> Display {
        Self::display(self.screen == MenuScreen::NetworkLobby)
    }

    pub(super) fn authors_popup_display(&self) -> Display {
        Self::display(self.screen == MenuScreen::ModeSelect && self.show_authors_popup)
    }

    pub(super) fn rules_popup_display(&self) -> Display {
        Self::display(self.screen == MenuScreen::ModeSelect && self.show_rules_popup)
    }

    pub(super) fn settings_popup_display(&self) -> Display {
        Self::display(self.screen == MenuScreen::ModeSelect && self.show_settings_popup)
    }

    pub(super) fn local_setup_display(&self) -> Display {
        Self::display(self.uses_setup_layout())
    }

    pub(super) fn network_setup_display(&self) -> Display {
        Self::display(self.screen == MenuScreen::Setup && self.game_mode == StartGameMode::Network)
    }

    pub(super) fn player_row_display(&self, player_index: usize) -> Display {
        Self::display(self.uses_setup_layout() && player_index < self.player_count)
    }

    pub(super) fn player_ai_display(&self, player_index: usize) -> Display {
        Self::display(
            self.uses_setup_layout()
                && player_index < self.player_count
                && self.player_controls[player_index].is_ai(),
        )
    }

    pub(super) fn detail_menu_display(
        &self,
        net_config: &crate::network::NetConfig,
        player_index: usize,
    ) -> Display {
        Self::display(
            self.open_player_detail_dropdown == Some(player_index)
                && self.screen == MenuScreen::Setup
                && player_index < self.player_count
                && self.player_detail_dropdown_enabled(net_config, player_index),
        )
    }

    pub(super) fn detail_option_display(
        &self,
        net_config: &crate::network::NetConfig,
        player_index: usize,
        option: PlayerDetailOption,
    ) -> Display {
        Self::display(
            self.open_player_detail_dropdown == Some(player_index)
                && self.screen == MenuScreen::Setup
                && player_index < self.player_count
                && self.player_detail_option_enabled(net_config, player_index, option),
        )
    }

    pub(super) fn lobby_host_only_display(&self, net_mode: NetMode) -> Display {
        Self::display(self.screen == MenuScreen::NetworkLobby && matches!(net_mode, NetMode::Host))
    }

    pub(super) fn lobby_client_only_display(&self, net_mode: NetMode) -> Display {
        Self::display(
            self.screen == MenuScreen::NetworkLobby && matches!(net_mode, NetMode::Client),
        )
    }

    pub(super) fn can_edit_full_lobby(&self, selected_net_mode: NetMode) -> bool {
        self.game_mode == StartGameMode::Local
            || (self.game_mode == StartGameMode::Network
                && matches!(selected_net_mode, NetMode::Host))
    }

    pub(super) fn player_detail_dropdown_enabled(
        &self,
        net_config: &crate::network::NetConfig,
        player_index: usize,
    ) -> bool {
        if player_index >= self.player_count {
            return false;
        }
        if self.player_controls[player_index].is_ai() {
            return self.game_mode == StartGameMode::Local
                || (self.game_mode == StartGameMode::Network
                    && matches!(self.configured_network_mode(net_config), NetMode::Host));
        }
        self.game_mode == StartGameMode::Network
    }

    pub(super) fn player_detail_option_enabled(
        &self,
        net_config: &crate::network::NetConfig,
        player_index: usize,
        option: PlayerDetailOption,
    ) -> bool {
        if player_index >= self.player_count {
            return false;
        }

        if self.player_controls[player_index].is_ai() {
            if !(self.game_mode == StartGameMode::Local
                || (self.game_mode == StartGameMode::Network
                    && matches!(self.configured_network_mode(net_config), NetMode::Host)))
            {
                return false;
            }
            return matches!(
                option,
                PlayerDetailOption::Heuristic | PlayerDetailOption::AlphaBeta
            );
        }

        if self.game_mode != StartGameMode::Network {
            return false;
        }

        match self.configured_network_mode(net_config) {
            NetMode::Host => matches!(
                option,
                PlayerDetailOption::Host | PlayerDetailOption::Client
            ),
            NetMode::Client => matches!(option, PlayerDetailOption::Client),
            NetMode::Local => false,
        }
    }

    pub(super) fn configured_network_mode(
        &self,
        net_config: &crate::network::NetConfig,
    ) -> NetMode {
        if self.game_mode == StartGameMode::Network {
            self.net_mode
        } else {
            net_config.mode
        }
    }

    pub(super) fn start_game_label(&self, net_mode: NetMode) -> &'static str {
        if self.screen == MenuScreen::NetworkLobby {
            if matches!(net_mode, NetMode::Client) {
                "Waiting for Host"
            } else {
                "Start Network Game"
            }
        } else if self.game_mode == StartGameMode::Network && self.net_mode == NetMode::Client {
            "Waiting for Host"
        } else {
            "Start Game"
        }
    }

    pub(super) fn synced_game_config(&self) -> GameConfig {
        GameConfig {
            board_radius: self.board_radius,
            player_count: self.player_count,
            player_controls: self.player_controls,
            player_ai_strategies: self.player_ai_strategies,
            player_colors: self.player_colors,
            ai_cooldown_seconds: self.ai_cooldown_ms as f32 / 1_000.0,
        }
    }

    pub(super) fn local_game_config(&self) -> GameConfig {
        let mut config = self.synced_game_config();
        config.player_controls = DEFAULT_PLAYER_CONTROLS;
        config.player_ai_strategies = DEFAULT_AI_STRATEGIES;

        for player_index in 0..self.player_count {
            config.player_controls[player_index] = self.player_controls[player_index];
            config.player_ai_strategies[player_index] = self.player_ai_strategies[player_index];
        }

        config
    }

    pub(super) fn network_game_config(
        &self,
        host_slot: usize,
        remote_slots: &[usize],
    ) -> GameConfig {
        let mut config = self.synced_game_config();
        config.player_controls = [PlayerControl::RandomAi; 6];

        for player_index in 0..config.player_count {
            if player_index == host_slot || remote_slots.contains(&player_index) {
                config.player_controls[player_index] = PlayerControl::Human;
            } else {
                config.player_controls[player_index] = self.player_controls[player_index];
            }
        }

        config
    }

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
