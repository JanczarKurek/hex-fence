use bevy::app::AppExit;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::app_state::{AppPhase, GameConfig};
use crate::network::{NetConfig, NetMode, NetRuntime};
use crate::settings::{self, AppSettings};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsUiState::default())
            .insert_resource(MenuSelection::default())
            .add_systems(OnEnter(AppPhase::Menu), setup_start_menu)
            .add_systems(OnExit(AppPhase::Menu), cleanup_start_menu)
            .add_systems(
                Update,
                (
                    handle_mode_choice_buttons,
                    handle_back_to_mode_button,
                    handle_menu_option_buttons,
                    handle_network_connect_button,
                    handle_network_address_focus,
                    handle_network_address_typing,
                    handle_start_game_button,
                    sync_menu_layout_visibility,
                    sync_menu_button_visuals,
                )
                    .run_if(in_state(AppPhase::Menu)),
            )
            .add_systems(OnEnter(AppPhase::InGame), setup_in_game_ui)
            .add_systems(
                Update,
                (
                    handle_exit_button,
                    handle_settings_toggle_button,
                    handle_tab_buttons,
                    handle_sound_slider_input,
                    sync_sound_slider_visuals,
                    sync_settings_popup_visibility,
                )
                    .run_if(in_state(AppPhase::InGame)),
            );
    }
}

#[derive(Resource)]
struct MenuSelection {
    screen: MenuScreen,
    game_mode: StartGameMode,
    board_radius: i32,
    player_count: usize,
    net_mode: NetMode,
    net_address: String,
    address_focused: bool,
}

impl Default for MenuSelection {
    fn default() -> Self {
        Self {
            screen: MenuScreen::ModeSelect,
            game_mode: StartGameMode::Local,
            board_radius: 4,
            player_count: 3,
            net_mode: NetMode::Local,
            net_address: "127.0.0.1:4000".to_string(),
            address_focused: false,
        }
    }
}

#[derive(Component)]
struct StartMenuRoot;

#[derive(Component)]
struct StartGameButton;

#[derive(Component)]
struct StartGameButtonLabel;

#[derive(Component)]
struct BoardSizeButton {
    radius: i32,
}

#[derive(Component)]
struct PlayerCountButton {
    player_count: usize,
}

#[derive(Component)]
struct ModeChoiceButton {
    mode: StartGameMode,
}

#[derive(Component)]
struct BackToModeButton;

#[derive(Component)]
struct MenuScreenModeSelect;

#[derive(Component)]
struct MenuScreenSetup;

#[derive(Component)]
struct LocalOnly;

#[derive(Component)]
struct NetworkOnly;

#[derive(Component)]
struct NetworkModeButton {
    mode: NetMode,
}

#[derive(Component)]
struct NetworkAddressInputButton;

#[derive(Component)]
struct NetworkAddressText;

#[derive(Component)]
struct NetworkConnectButton;

#[derive(Component)]
struct ConnectedPlayersText;

#[derive(Clone, Copy, Eq, PartialEq)]
enum MenuScreen {
    ModeSelect,
    Setup,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum StartGameMode {
    Local,
    Network,
}

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct SettingsToggleButton;

#[derive(Component)]
struct SettingsPopup;

#[derive(Component)]
struct SettingsTabButton {
    tab: SettingsTab,
}

#[derive(Component)]
struct SettingsTabContent {
    tab: SettingsTab,
}

#[derive(Component)]
struct SoundSliderTrack {
    kind: SoundSliderKind,
}

#[derive(Component)]
struct SoundSliderFill {
    kind: SoundSliderKind,
}

#[derive(Component)]
struct SoundSliderValueText {
    kind: SoundSliderKind,
}

#[derive(Resource)]
struct SettingsUiState {
    open: bool,
    active_tab: SettingsTab,
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
enum SettingsTab {
    Sound,
}

#[derive(Component, Clone, Copy)]
enum SoundSliderKind {
    Master,
    Music,
    Effects,
}

impl SoundSliderKind {
    fn value(self, settings: &AppSettings) -> f32 {
        match self {
            Self::Master => settings.audio.master,
            Self::Music => settings.audio.music,
            Self::Effects => settings.audio.effects,
        }
    }

    fn set_value(self, settings: &mut AppSettings, value: f32) {
        let value = value.clamp(0.0, 1.0);
        match self {
            Self::Master => settings.audio.master = value,
            Self::Music => settings.audio.music = value,
            Self::Effects => settings.audio.effects = value,
        }
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.8, 0.2, 0.2);
const MENU_PANEL_BG: Color = Color::srgba(0.06, 0.07, 0.08, 0.95);
const MENU_SELECTED: Color = Color::srgb(0.20, 0.58, 0.36);
const MENU_START: Color = Color::srgb(0.23, 0.62, 0.40);
const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.1, 0.95);
const TAB_ACTIVE: Color = Color::srgb(0.22, 0.33, 0.44);
const TAB_INACTIVE: Color = Color::srgb(0.13, 0.13, 0.15);
const SLIDER_TRACK: Color = Color::srgb(0.18, 0.18, 0.2);
const SLIDER_FILL: Color = Color::srgb(0.25, 0.68, 0.44);

fn setup_start_menu(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    net_config: Res<NetConfig>,
    mut menu: ResMut<MenuSelection>,
) {
    menu.screen = MenuScreen::ModeSelect;
    menu.game_mode = if matches!(net_config.mode, NetMode::Local) {
        StartGameMode::Local
    } else {
        StartGameMode::Network
    };
    menu.board_radius = game_config.board_radius;
    menu.player_count = game_config.player_count;
    menu.net_mode = net_config.mode;
    menu.net_address = net_config.address.clone();
    menu.address_focused = false;

    commands
        .spawn((
            StartMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(480.0),
                    max_width: Val::Percent(92.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    row_gap: Val::Px(18.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BorderColor(Color::srgb(0.22, 0.22, 0.25)),
                BackgroundColor(MENU_PANEL_BG),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Hex Board Setup"),
                    TextFont::from_font_size(34.0),
                    TextColor(Color::WHITE),
                ));

                panel
                    .spawn((
                        MenuScreenModeSelect,
                        Node {
                            width: Val::Percent(100.0),
                            row_gap: Val::Px(12.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ))
                    .with_children(|step| {
                        step.spawn((
                            Text::new("Choose Mode"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        step.spawn(Node {
                            width: Val::Percent(100.0),
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                ModeChoiceButton {
                                    mode: StartGameMode::Local,
                                },
                                Node {
                                    width: Val::Px(220.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor(Color::BLACK),
                                BackgroundColor(NORMAL_BUTTON),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("Local Multiplayer"),
                                    TextFont::from_font_size(18.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                ModeChoiceButton {
                                    mode: StartGameMode::Network,
                                },
                                Node {
                                    width: Val::Px(240.0),
                                    height: Val::Px(44.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor(Color::BLACK),
                                BackgroundColor(NORMAL_BUTTON),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("Network Multiplayer"),
                                    TextFont::from_font_size(18.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                    });

                panel
                    .spawn((
                        MenuScreenSetup,
                        Node {
                            width: Val::Percent(100.0),
                            row_gap: Val::Px(12.0),
                            flex_direction: FlexDirection::Column,
                            display: Display::None,
                            ..default()
                        },
                    ))
                    .with_children(|step| {
                        step.spawn((
                            Button,
                            BackToModeButton,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Back"),
                                TextFont::from_font_size(16.0),
                                TextColor(Color::WHITE),
                            ));
                        });

                        step.spawn((
                            Text::new("Board Size"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        spawn_choice_row(step, &[3, 4, 5, 6], menu.board_radius);

                        step.spawn((
                            LocalOnly,
                            Text::new("Players"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        step.spawn((LocalOnly, Node::default()))
                            .with_children(|local| {
                                spawn_player_row(local, &[2, 3, 6], menu.player_count);
                            });

                        step.spawn((
                            NetworkOnly,
                            Text::new("Role"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        step.spawn((NetworkOnly, Node::default()))
                            .with_children(|network| {
                                spawn_network_mode_row(network, menu.net_mode);
                            });

                        step.spawn((
                            NetworkOnly,
                            Text::new("Server Address"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        step.spawn((NetworkOnly, Node::default()))
                            .with_children(|network| {
                                network
                                    .spawn((
                                        Button,
                                        NetworkAddressInputButton,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(38.0),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::Center,
                                            padding: UiRect::horizontal(Val::Px(10.0)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BorderColor(Color::BLACK),
                                        BackgroundColor(NORMAL_BUTTON),
                                    ))
                                    .with_children(|input| {
                                        input.spawn((
                                            NetworkAddressText,
                                            Text::new(menu.net_address.clone()),
                                            TextFont::from_font_size(16.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });

                        step.spawn((NetworkOnly, Node::default()))
                            .with_children(|network| {
                                network
                                    .spawn((
                                        Button,
                                        NetworkConnectButton,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(40.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BorderColor(Color::BLACK),
                                        BackgroundColor(NORMAL_BUTTON),
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Text::new("Apply Network Settings"),
                                            TextFont::from_font_size(16.0),
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            });

                        step.spawn((
                            NetworkOnly,
                            Text::new("Connected Players"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        step.spawn((
                            ConnectedPlayersText,
                            NetworkOnly,
                            Text::new(""),
                            TextFont::from_font_size(16.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));

                        step.spawn((
                            Button,
                            StartGameButton,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(48.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                margin: UiRect::top(Val::Px(8.0)),
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BackgroundColor(MENU_START),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                StartGameButtonLabel,
                                Text::new("Start Game"),
                                TextFont::from_font_size(24.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
            });
        });
}

fn spawn_choice_row(parent: &mut ChildSpawnerCommands, choices: &[i32], selected: i32) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(8.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for choice in choices {
                row.spawn((
                    Button,
                    BoardSizeButton { radius: *choice },
                    Node {
                        width: Val::Px(72.0),
                        height: Val::Px(38.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(if *choice == selected {
                        MENU_SELECTED
                    } else {
                        NORMAL_BUTTON
                    }),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(choice.to_string()),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

fn spawn_player_row(parent: &mut ChildSpawnerCommands, choices: &[usize], selected: usize) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(8.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for choice in choices {
                row.spawn((
                    Button,
                    PlayerCountButton {
                        player_count: *choice,
                    },
                    Node {
                        width: Val::Px(72.0),
                        height: Val::Px(38.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(if *choice == selected {
                        MENU_SELECTED
                    } else {
                        NORMAL_BUTTON
                    }),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(choice.to_string()),
                        TextFont::from_font_size(18.0),
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

fn spawn_network_mode_row(parent: &mut ChildSpawnerCommands, selected: NetMode) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(8.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|row| {
            for (label, mode) in [("Host", NetMode::Host), ("Client", NetMode::Client)] {
                row.spawn((
                    Button,
                    NetworkModeButton { mode },
                    Node {
                        width: Val::Px(96.0),
                        height: Val::Px(38.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor(Color::BLACK),
                    BackgroundColor(if mode == selected {
                        MENU_SELECTED
                    } else {
                        NORMAL_BUTTON
                    }),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(label),
                        TextFont::from_font_size(16.0),
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
}

fn cleanup_start_menu(mut commands: Commands, roots: Query<Entity, With<StartMenuRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

fn handle_mode_choice_buttons(
    mut menu: ResMut<MenuSelection>,
    mut interactions: Query<
        (&Interaction, &ModeChoiceButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            menu.game_mode = button.mode;
            menu.screen = MenuScreen::Setup;
            menu.address_focused = false;
            if menu.game_mode == StartGameMode::Network && matches!(menu.net_mode, NetMode::Local) {
                menu.net_mode = NetMode::Host;
            }
        }
    }
}

fn handle_back_to_mode_button(
    mut menu: ResMut<MenuSelection>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToModeButton>),
    >,
) {
    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                menu.screen = MenuScreen::ModeSelect;
                menu.address_focused = false;
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}

fn handle_menu_option_buttons(
    mut menu: ResMut<MenuSelection>,
    mut board_buttons: Query<
        (&Interaction, &BoardSizeButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<BoardSizeButton>,
            Without<PlayerCountButton>,
        ),
    >,
    mut player_buttons: Query<
        (&Interaction, &PlayerCountButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<PlayerCountButton>,
            Without<BoardSizeButton>,
        ),
    >,
    mut network_mode_buttons: Query<
        (&Interaction, &NetworkModeButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<NetworkModeButton>,
            Without<BoardSizeButton>,
            Without<PlayerCountButton>,
        ),
    >,
) {
    if menu.screen != MenuScreen::Setup {
        return;
    }

    for (interaction, button) in &mut board_buttons {
        if *interaction == Interaction::Pressed {
            menu.board_radius = button.radius;
        }
    }

    for (interaction, button) in &mut player_buttons {
        if *interaction == Interaction::Pressed {
            menu.player_count = button.player_count;
        }
    }

    for (interaction, button) in &mut network_mode_buttons {
        if *interaction == Interaction::Pressed {
            menu.net_mode = button.mode;
        }
    }
}

fn handle_network_connect_button(
    menu: Res<MenuSelection>,
    mut net_config: ResMut<NetConfig>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<NetworkConnectButton>),
    >,
) {
    if menu.screen != MenuScreen::Setup || menu.game_mode != StartGameMode::Network {
        return;
    }

    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                net_config.mode = menu.net_mode;
                net_config.local_player_index = local_player_index_for_mode(menu.net_mode);
                let trimmed = menu.net_address.trim();
                net_config.address = if trimmed.is_empty() {
                    "127.0.0.1:4000".to_string()
                } else {
                    trimmed.to_string()
                };
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}

fn handle_network_address_focus(
    mut menu: ResMut<MenuSelection>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<NetworkAddressInputButton>),
    >,
) {
    if menu.screen != MenuScreen::Setup || menu.game_mode != StartGameMode::Network {
        return;
    }

    let mut clicked_input = false;
    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                clicked_input = true;
                *color = MENU_SELECTED.into();
            }
            Interaction::Hovered if !menu.address_focused => *color = HOVERED_BUTTON.into(),
            Interaction::None if !menu.address_focused => *color = NORMAL_BUTTON.into(),
            _ => {}
        }
    }

    if clicked_input {
        menu.address_focused = true;
    }
}

fn handle_network_address_typing(
    mut menu: ResMut<MenuSelection>,
    mut key_events: EventReader<KeyboardInput>,
) {
    if menu.screen != MenuScreen::Setup || menu.game_mode != StartGameMode::Network {
        key_events.clear();
        return;
    }

    if !menu.address_focused {
        key_events.clear();
        return;
    }

    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            KeyCode::Backspace => {
                menu.net_address.pop();
                continue;
            }
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Escape => {
                menu.address_focused = false;
                continue;
            }
            _ => {}
        }

        if let Key::Character(text) = &event.logical_key {
            for ch in text.chars() {
                if is_valid_address_char(ch) && menu.net_address.len() < 80 {
                    menu.net_address.push(ch);
                }
            }
        }
    }
}

fn is_valid_address_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '-')
}

fn sync_menu_layout_visibility(
    menu: Res<MenuSelection>,
    mut sections: Query<
        (
            Option<&MenuScreenModeSelect>,
            Option<&MenuScreenSetup>,
            Option<&LocalOnly>,
            Option<&NetworkOnly>,
            &mut Node,
        ),
    >,
) {
    if !menu.is_changed() {
        return;
    }

    for (mode_screen, setup_screen, local_only, network_only, mut node) in &mut sections {
        if mode_screen.is_some() {
            node.display = if menu.screen == MenuScreen::ModeSelect {
                Display::Flex
            } else {
                Display::None
            };
        } else if setup_screen.is_some() {
            node.display = if menu.screen == MenuScreen::Setup {
                Display::Flex
            } else {
                Display::None
            };
        } else if local_only.is_some() {
            node.display = if menu.screen == MenuScreen::Setup && menu.game_mode == StartGameMode::Local
            {
                Display::Flex
            } else {
                Display::None
            };
        } else if network_only.is_some() {
            node.display =
                if menu.screen == MenuScreen::Setup && menu.game_mode == StartGameMode::Network {
                    Display::Flex
                } else {
                    Display::None
                };
        }
    }
}

fn sync_menu_button_visuals(
    menu: Res<MenuSelection>,
    net_runtime: Res<NetRuntime>,
    mut option_buttons: Query<
        (
            &Interaction,
            Option<&BoardSizeButton>,
            Option<&PlayerCountButton>,
            Option<&ModeChoiceButton>,
            Option<&NetworkModeButton>,
            Option<&NetworkAddressInputButton>,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
    mut menu_texts: Query<
        (
            Option<&NetworkAddressText>,
            Option<&ConnectedPlayersText>,
            Option<&StartGameButtonLabel>,
            &mut Text,
        ),
    >,
) {
    for (interaction, board, player, mode_choice, network_mode, address_input, mut color) in
        &mut option_buttons
    {
        *color = if let Some(button) = board {
            if button.radius == menu.board_radius {
                MENU_SELECTED.into()
            } else if *interaction == Interaction::Hovered {
                HOVERED_BUTTON.into()
            } else {
                NORMAL_BUTTON.into()
            }
        } else if let Some(button) = player {
            if button.player_count == menu.player_count {
                MENU_SELECTED.into()
            } else if *interaction == Interaction::Hovered {
                HOVERED_BUTTON.into()
            } else {
                NORMAL_BUTTON.into()
            }
        } else if let Some(button) = mode_choice {
            if button.mode == menu.game_mode {
                MENU_SELECTED.into()
            } else if *interaction == Interaction::Hovered {
                HOVERED_BUTTON.into()
            } else {
                NORMAL_BUTTON.into()
            }
        } else if let Some(button) = network_mode {
            if button.mode == menu.net_mode {
                MENU_SELECTED.into()
            } else if *interaction == Interaction::Hovered {
                HOVERED_BUTTON.into()
            } else {
                NORMAL_BUTTON.into()
            }
        } else if address_input.is_some() {
            if menu.address_focused {
                MENU_SELECTED.into()
            } else if *interaction == Interaction::Hovered {
                HOVERED_BUTTON.into()
            } else {
                NORMAL_BUTTON.into()
            }
        } else {
            *color
        };
    }

    for (address_text, connected_text, start_text, mut text) in &mut menu_texts {
        if address_text.is_some() {
            let mut label = menu.net_address.clone();
            if menu.address_focused {
                label.push('_');
            }
            if menu.net_mode == NetMode::Client {
                label.push_str(if net_runtime.connected {
                    "  (connected)"
                } else {
                    "  (not connected)"
                });
            }
            *text = Text::new(label);
        } else if connected_text.is_some() {
            *text = Text::new(connected_players_label(
                menu.game_mode,
                menu.net_mode,
                net_runtime.connected,
            ));
        } else if start_text.is_some() {
            let label = if menu.game_mode == StartGameMode::Network
                && menu.net_mode == NetMode::Client
            {
                "Waiting for Host"
            } else {
                "Start Game"
            };
            *text = Text::new(label);
        }
    }
}

fn handle_start_game_button(
    menu: Res<MenuSelection>,
    mut net_config: ResMut<NetConfig>,
    mut game_config: ResMut<GameConfig>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartGameButton>),
    >,
    mut next_phase: ResMut<NextState<AppPhase>>,
) {
    if menu.screen != MenuScreen::Setup {
        return;
    }

    for (interaction, mut color) in &mut interactions {
        if menu.game_mode == StartGameMode::Network && matches!(menu.net_mode, NetMode::Client) {
            *color = NORMAL_BUTTON.into();
            continue;
        }

        match *interaction {
            Interaction::Pressed => {
                net_config.mode = if menu.game_mode == StartGameMode::Local {
                    NetMode::Local
                } else {
                    menu.net_mode
                };
                net_config.local_player_index = local_player_index_for_mode(net_config.mode);
                let trimmed = menu.net_address.trim();
                net_config.address = if trimmed.is_empty() {
                    "127.0.0.1:4000".to_string()
                } else {
                    trimmed.to_string()
                };
                game_config.board_radius = menu.board_radius;
                game_config.player_count = if menu.game_mode == StartGameMode::Network {
                    2
                } else {
                    menu.player_count
                };
                next_phase.set(AppPhase::InGame);
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = MENU_START.into(),
        }
    }
}

fn local_player_index_for_mode(mode: NetMode) -> usize {
    if matches!(mode, NetMode::Client) {
        1
    } else {
        0
    }
}

fn connected_players_label(game_mode: StartGameMode, net_mode: NetMode, connected: bool) -> String {
    if game_mode != StartGameMode::Network {
        return "Not used in local mode.".to_string();
    }

    match net_mode {
        NetMode::Host => format!(
            "1. Player 1 (Host) - You\n2. Player 2 (Client) - {}",
            if connected { "Connected" } else { "Waiting" }
        ),
        NetMode::Client => format!(
            "1. Player 1 (Host) - {}\n2. Player 2 (Client) - You",
            if connected { "Connected" } else { "Waiting" }
        ),
        NetMode::Local => "Choose Host or Client.".to_string(),
    }
}

fn setup_in_game_ui(
    mut commands: Commands,
    app_settings: Res<AppSettings>,
    settings_ui: Res<SettingsUiState>,
) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Auto,
                    height: Val::Auto,
                    position_type: PositionType::Absolute,
                    top: Val::Px(12.0),
                    right: Val::Px(12.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|top_buttons| {
                    top_buttons
                        .spawn((
                            Button,
                            SettingsToggleButton,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(44.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Settings"),
                                TextFont::from_font_size(18.0),
                                TextColor(Color::WHITE),
                            ));
                        });

                    top_buttons
                        .spawn((
                            Button,
                            ExitButton,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(44.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Exit"),
                                TextFont::from_font_size(18.0),
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            parent
                .spawn((
                    SettingsPopup,
                    Node {
                        width: Val::Px(560.0),
                        max_width: Val::Percent(96.0),
                        height: Val::Px(360.0),
                        max_height: Val::Percent(90.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Stretch,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        display: if settings_ui.open {
                            Display::Flex
                        } else {
                            Display::None
                        },
                        ..default()
                    },
                    BorderColor(Color::srgb(0.24, 0.24, 0.28)),
                    BackgroundColor(PANEL_BG),
                    Transform::from_translation(Vec3::new(-280.0, -180.0, 0.0)),
                ))
                .with_children(|popup| {
                    popup
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(40.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|tabs| {
                            tabs.spawn((
                                Button,
                                SettingsTabButton {
                                    tab: SettingsTab::Sound,
                                },
                                Node {
                                    width: Val::Px(140.0),
                                    height: Val::Percent(100.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(TAB_ACTIVE),
                            ))
                            .with_children(|tab| {
                                tab.spawn((
                                    Text::new("Sound"),
                                    TextFont::from_font_size(16.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    popup
                        .spawn((
                            SettingsTabContent {
                                tab: SettingsTab::Sound,
                            },
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(20.0),
                                ..default()
                            },
                        ))
                        .with_children(|content| {
                            content.spawn((
                                Text::new("Audio Mix"),
                                TextFont::from_font_size(22.0),
                                TextColor(Color::WHITE),
                            ));

                            spawn_sound_slider_row(
                                content,
                                "Master Volume",
                                SoundSliderKind::Master,
                                app_settings.audio.master,
                            );
                            spawn_sound_slider_row(
                                content,
                                "Music Volume",
                                SoundSliderKind::Music,
                                app_settings.audio.music,
                            );
                            spawn_sound_slider_row(
                                content,
                                "Effects Volume",
                                SoundSliderKind::Effects,
                                app_settings.audio.effects,
                            );
                        });
                });
        });
}

fn spawn_sound_slider_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    kind: SoundSliderKind,
    value: f32,
) {
    let clamped = value.clamp(0.0, 1.0);
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(42.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(16.0),
                TextColor(Color::WHITE),
                Node {
                    width: Val::Px(120.0),
                    ..default()
                },
            ));

            row.spawn((
                Button,
                RelativeCursorPosition::default(),
                SoundSliderTrack { kind },
                Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(300.0),
                    height: Val::Px(22.0),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Stretch,
                    ..default()
                },
                BorderColor(Color::srgb(0.35, 0.35, 0.4)),
                BackgroundColor(SLIDER_TRACK),
            ))
            .with_children(|track| {
                track.spawn((
                    SoundSliderFill { kind },
                    Node {
                        width: Val::Percent(clamped * 100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SLIDER_FILL),
                ));
            });

            row.spawn((
                SoundSliderValueText { kind },
                Text::new(format!("{:>3}%", (clamped * 100.0).round() as i32)),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.86, 0.86, 0.9)),
                Node {
                    width: Val::Px(56.0),
                    ..default()
                },
            ));
        });
}

fn handle_exit_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ExitButton>),
    >,
    mut exit_events: EventWriter<AppExit>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                exit_events.write(AppExit::Success);
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}

fn handle_settings_toggle_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsToggleButton>),
    >,
    mut settings_ui: ResMut<SettingsUiState>,
    app_settings: Res<AppSettings>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let was_open = settings_ui.open;
                settings_ui.open = !settings_ui.open;
                if was_open && !settings_ui.open {
                    let _ = settings::save_settings_to_disk(*app_settings);
                }
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}

fn handle_tab_buttons(
    mut tab_interactions: Query<
        (&Interaction, &SettingsTabButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings_ui: ResMut<SettingsUiState>,
) {
    for (interaction, tab_button) in &mut tab_interactions {
        if *interaction == Interaction::Pressed {
            settings_ui.active_tab = tab_button.tab;
        }
    }
}

fn sync_settings_popup_visibility(
    settings_ui: Res<SettingsUiState>,
    mut popup_query: Query<&mut Node, With<SettingsPopup>>,
    mut tab_button_query: Query<(&SettingsTabButton, &mut BackgroundColor), With<Button>>,
    mut tab_content_query: Query<(&SettingsTabContent, &mut Node), Without<SettingsPopup>>,
) {
    if !settings_ui.is_changed() {
        return;
    }

    if let Ok(mut popup_node) = popup_query.single_mut() {
        popup_node.display = if settings_ui.open {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (tab_button, mut tab_color) in &mut tab_button_query {
        *tab_color = if tab_button.tab == settings_ui.active_tab {
            TAB_ACTIVE.into()
        } else {
            TAB_INACTIVE.into()
        };
    }

    for (tab_content, mut tab_node) in &mut tab_content_query {
        tab_node.display = if tab_content.tab == settings_ui.active_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn handle_sound_slider_input(
    mut app_settings: ResMut<AppSettings>,
    track_query: Query<(&Interaction, &RelativeCursorPosition, &SoundSliderTrack), With<Button>>,
) {
    for (interaction, cursor_pos, slider) in &track_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(normalized) = cursor_pos.normalized else {
            continue;
        };

        slider.kind.set_value(&mut app_settings, normalized.x);
    }
}

fn sync_sound_slider_visuals(
    app_settings: Res<AppSettings>,
    mut fill_query: Query<(&SoundSliderFill, &mut Node)>,
    mut value_text_query: Query<(&SoundSliderValueText, &mut Text)>,
) {
    if !app_settings.is_changed() {
        return;
    }

    for (fill, mut node) in &mut fill_query {
        node.width = Val::Percent(fill.kind.value(&app_settings) * 100.0);
    }

    for (value_text, mut text) in &mut value_text_query {
        let value = (value_text.kind.value(&app_settings) * 100.0).round() as i32;
        *text = Text::new(format!("{:>3}%", value));
    }
}
