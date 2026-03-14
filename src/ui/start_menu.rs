use bevy::app::AppExit;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy_simple_text_input::{TextInput, TextInputInactive, TextInputValue};

use crate::app_state::{AiStrategy, AppPhase, GameConfig, PlayerColor, PlayerControl};
use crate::network::{NetConfig, NetLobbyState, NetMode, NetRuntime, NetUiCommand};
use crate::settings::{self, AppSettings, LastNetMode};

use crate::game::despawn_all;

use super::components::{
    AiCooldownButton, AuthorsPopup, BackToModeButton, BoardSizeButton, ConnectedPlayersText,
    ControlBindingButton, ControlBindingKind, ControlBindingValueText, LobbyPlayerListScroll,
    LocalOnly, MainMenuAction, MainMenuActionButton, MenuMainPanel, MenuScreen,
    MenuScreenModeSelect, MenuScreenNetworkLobby, MenuScreenSetup, MenuSelection,
    MenuSettingsCloseButton, MenuSettingsPopup, NetworkAddressInputButton,
    NetworkAddressInputField, NetworkConnectButton, NetworkLobbyClientOnly, NetworkLobbyHostOnly,
    NetworkModeButton, NetworkOnly, NetworkSlotButton, NetworkSlotOwner, PlayerAiOnly,
    PlayerColorButton, PlayerControlButton, PlayerControlToggleButton, PlayerCountButton,
    PlayerDetailDropdownButton, PlayerDetailDropdownMenu, PlayerDetailDropdownText,
    PlayerDetailOption, PlayerDetailOptionButton, PlayerSetupRow, RulesPopup, SettingsTab,
    SettingsTabButton, SettingsTabContent, SettingsUiState, SoundSliderFill, SoundSliderKind,
    SoundSliderTrack, SoundSliderValueText, StartGameButton, StartGameButtonLabel, StartGameMode,
    StartMenuRoot,
};
use super::styles::{
    HOVERED_BUTTON, MENU_PANEL_BG, MENU_SELECTED, MENU_START, NORMAL_BUTTON, PRESSED_BUTTON,
    TAB_ACTIVE, TAB_INACTIVE, button_bundle, button_node, menu_text, neutral_button_color,
    selected_button_color, white_text,
};
use super::widgets::{
    spawn_ai_cooldown_row, spawn_choice_row, spawn_control_binding_row, spawn_network_mode_row,
    spawn_player_row, spawn_sound_slider_row,
};

const AI_COOLDOWN_CHOICES_MS: [u32; 5] = [250, 500, 1_000, 1_500, 2_000];

pub(super) fn setup_start_menu(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    app_settings: Res<AppSettings>,
    net_config: Res<NetConfig>,
    net_lobby: Res<NetLobbyState>,
    mut settings_ui: ResMut<SettingsUiState>,
    mut menu: ResMut<MenuSelection>,
) {
    settings_ui.pending_control_binding = None;
    menu.screen = MenuScreen::ModeSelect;
    menu.game_mode = if matches!(net_config.mode, NetMode::Local) {
        StartGameMode::Local
    } else {
        StartGameMode::Network
    };
    menu.board_radius = game_config.board_radius;
    menu.player_count = game_config.player_count;
    menu.player_controls = game_config.player_controls;
    menu.player_ai_strategies = game_config.player_ai_strategies;
    menu.player_colors = game_config.player_colors;
    menu.ai_cooldown_ms = nearest_ai_cooldown_ms(game_config.ai_cooldown_seconds);
    menu.net_mode = net_config.mode;
    menu.net_address = net_config.address.clone();
    menu.network_local_slot = if matches!(net_config.mode, NetMode::Host) {
        net_lobby.host_slot
    } else {
        net_lobby.client_slot
    };
    menu.address_focused = false;
    menu.show_authors_popup = false;
    menu.show_rules_popup = false;
    menu.show_settings_popup = false;
    menu.open_player_detail_dropdown = None;

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
                MenuMainPanel,
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
                panel.spawn(white_text("Hex Fence", 42.0));

                panel
                    .spawn((
                        MenuScreenModeSelect,
                        Node {
                            width: Val::Percent(100.0),
                            row_gap: Val::Px(12.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|step| {
                        for (label, action) in [
                            ("Local Game", MainMenuAction::LocalGame),
                            ("Network Game", MainMenuAction::NetworkGame),
                            ("Settings", MainMenuAction::Settings),
                            ("Rules", MainMenuAction::Rules),
                            ("Authors", MainMenuAction::Authors),
                            ("Quit", MainMenuAction::Quit),
                        ] {
                            step.spawn(button_bundle(
                                MainMenuActionButton { action },
                                button_node(260.0, 46.0, 1.0),
                                NORMAL_BUTTON,
                            ))
                            .with_children(|button| {
                                button.spawn(white_text(label, 18.0));
                            });
                        }
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
                        step.spawn(button_bundle(
                            BackToModeButton,
                            button_node(120.0, 36.0, 1.0),
                            NORMAL_BUTTON,
                        ))
                        .with_children(|button| {
                            button.spawn(white_text("Back", 16.0));
                        });

                        step.spawn(menu_text("Board Size", 20.0));
                        spawn_choice_row(step, &[3, 4, 5, 6], menu.board_radius);

                        step.spawn((LocalOnly, menu_text("Lobby Setup", 20.0)));
                        step.spawn((
                            LocalOnly,
                            Node {
                                width: Val::Percent(100.0),
                                column_gap: Val::Px(12.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::FlexStart,
                                ..default()
                            },
                        ))
                        .with_children(|local| {
                            local
                                .spawn((
                                    LobbyPlayerListScroll,
                                    ScrollPosition::default(),
                                    RelativeCursorPosition::default(),
                                    Node {
                                        width: Val::Percent(62.0),
                                        height: Val::Px(430.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(6.0),
                                        overflow: Overflow::scroll_y(),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.10, 0.11, 0.13, 0.8)),
                                ))
                                .with_children(|players| {
                                    players
                                        .spawn((Node {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::bottom(Val::Px(6.0)),
                                            ..default()
                                        },))
                                        .with_children(|header| {
                                            header.spawn(menu_text("Players", 17.0));
                                        });

                                    for player_index in 0..6 {
                                        players
                                            .spawn((
                                                PlayerSetupRow { player_index },
                                                Node {
                                                    width: Val::Percent(100.0),
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: Val::Px(6.0),
                                                    padding: UiRect::all(Val::Px(8.0)),
                                                    border: UiRect::all(Val::Px(1.0)),
                                                    ..default()
                                                },
                                                BorderColor(Color::srgba(0.95, 0.95, 1.0, 0.1)),
                                                BackgroundColor(Color::srgba(
                                                    0.06, 0.07, 0.09, 0.92,
                                                )),
                                            ))
                                            .with_children(|row| {
                                                row.spawn(menu_text(
                                                    format!("Player {}", player_index + 1),
                                                    15.0,
                                                ));

                                                row.spawn(Node {
                                                    width: Val::Percent(100.0),
                                                    flex_direction: FlexDirection::Row,
                                                    column_gap: Val::Px(6.0),
                                                    ..default()
                                                })
                                                .with_children(|controls| {
                                                    controls
                                                        .spawn(button_bundle(
                                                            PlayerControlToggleButton {
                                                                player_index,
                                                            },
                                                            button_node(86.0, 30.0, 1.0),
                                                            NORMAL_BUTTON,
                                                        ))
                                                        .with_children(|button| {
                                                            button.spawn((
                                                                PlayerControlButton {
                                                                    player_index,
                                                                },
                                                                white_text("Human", 14.0),
                                                            ));
                                                        });
                                                    controls
                                                        .spawn(button_bundle(
                                                            PlayerDetailDropdownButton {
                                                                player_index,
                                                            },
                                                            Node {
                                                                width: Val::Percent(100.0),
                                                                min_height: Val::Px(30.0),
                                                                justify_content:
                                                                    JustifyContent::FlexStart,
                                                                align_items: AlignItems::Center,
                                                                padding: UiRect::horizontal(
                                                                    Val::Px(10.0),
                                                                ),
                                                                border: UiRect::all(Val::Px(1.0)),
                                                                ..default()
                                                            },
                                                            NORMAL_BUTTON,
                                                        ))
                                                        .with_children(|button| {
                                                            button.spawn((
                                                                PlayerDetailDropdownText {
                                                                    player_index,
                                                                },
                                                                white_text("Details", 13.0),
                                                            ));
                                                        });
                                                });

                                                row.spawn((
                                                    PlayerDetailDropdownMenu { player_index },
                                                    Node {
                                                        position_type: PositionType::Absolute,
                                                        left: Val::Px(92.0),
                                                        right: Val::Px(0.0),
                                                        top: Val::Px(42.0),
                                                        flex_direction: FlexDirection::Column,
                                                        row_gap: Val::Px(2.0),
                                                        padding: UiRect::all(Val::Px(4.0)),
                                                        border: UiRect::all(Val::Px(1.0)),
                                                        display: Display::None,
                                                        ..default()
                                                    },
                                                    BorderColor(Color::srgba(0.9, 0.9, 0.95, 0.35)),
                                                    BackgroundColor(Color::srgba(0.06, 0.07, 0.1, 0.98)),
                                                    ZIndex(20),
                                                ))
                                                .with_children(|menu| {
                                                    menu.spawn(button_bundle(
                                                        PlayerDetailOptionButton {
                                                            player_index,
                                                            option: PlayerDetailOption::Host,
                                                        },
                                                        Node {
                                                            width: Val::Percent(100.0),
                                                            height: Val::Px(26.0),
                                                            justify_content:
                                                                JustifyContent::FlexStart,
                                                            align_items: AlignItems::Center,
                                                            padding: UiRect::horizontal(Val::Px(8.0)),
                                                            border: UiRect::all(Val::Px(1.0)),
                                                            ..default()
                                                        },
                                                        NORMAL_BUTTON,
                                                    ))
                                                    .with_children(|button| {
                                                        button.spawn(white_text("Host", 12.0));
                                                    });
                                                    menu.spawn(button_bundle(
                                                        PlayerDetailOptionButton {
                                                            player_index,
                                                            option: PlayerDetailOption::Client,
                                                        },
                                                        Node {
                                                            width: Val::Percent(100.0),
                                                            height: Val::Px(26.0),
                                                            justify_content:
                                                                JustifyContent::FlexStart,
                                                            align_items: AlignItems::Center,
                                                            padding: UiRect::horizontal(Val::Px(8.0)),
                                                            border: UiRect::all(Val::Px(1.0)),
                                                            ..default()
                                                        },
                                                        NORMAL_BUTTON,
                                                    ))
                                                    .with_children(|button| {
                                                        button.spawn(white_text("Client", 12.0));
                                                    });
                                                    menu.spawn(button_bundle(
                                                        PlayerDetailOptionButton {
                                                            player_index,
                                                            option: PlayerDetailOption::Heuristic,
                                                        },
                                                        Node {
                                                            width: Val::Percent(100.0),
                                                            height: Val::Px(26.0),
                                                            justify_content:
                                                                JustifyContent::FlexStart,
                                                            align_items: AlignItems::Center,
                                                            padding: UiRect::horizontal(Val::Px(8.0)),
                                                            border: UiRect::all(Val::Px(1.0)),
                                                            ..default()
                                                        },
                                                        NORMAL_BUTTON,
                                                    ))
                                                    .with_children(|button| {
                                                        button.spawn(white_text("Heuristic", 12.0));
                                                    });
                                                    menu.spawn(button_bundle(
                                                        PlayerDetailOptionButton {
                                                            player_index,
                                                            option: PlayerDetailOption::AlphaBeta,
                                                        },
                                                        Node {
                                                            width: Val::Percent(100.0),
                                                            height: Val::Px(26.0),
                                                            justify_content:
                                                                JustifyContent::FlexStart,
                                                            align_items: AlignItems::Center,
                                                            padding: UiRect::horizontal(Val::Px(8.0)),
                                                            border: UiRect::all(Val::Px(1.0)),
                                                            ..default()
                                                        },
                                                        NORMAL_BUTTON,
                                                    ))
                                                    .with_children(|button| {
                                                        button.spawn(white_text("AlphaBeta", 12.0));
                                                    });
                                                });

                                                row.spawn(Node {
                                                    width: Val::Percent(100.0),
                                                    flex_direction: FlexDirection::Row,
                                                    column_gap: Val::Px(4.0),
                                                    flex_wrap: FlexWrap::Wrap,
                                                    ..default()
                                                })
                                                .with_children(|colors| {
                                                    for color in [
                                                        PlayerColor::Red,
                                                        PlayerColor::Blue,
                                                        PlayerColor::Gold,
                                                        PlayerColor::Teal,
                                                        PlayerColor::Pink,
                                                        PlayerColor::Orange,
                                                    ] {
                                                        colors
                                                            .spawn((
                                                                Button,
                                                                PlayerColorButton {
                                                                    player_index,
                                                                    color,
                                                                },
                                                                Node {
                                                                    width: Val::Px(28.0),
                                                                    height: Val::Px(26.0),
                                                                    justify_content:
                                                                        JustifyContent::Center,
                                                                    align_items: AlignItems::Center,
                                                                    border: UiRect::all(Val::Px(
                                                                        1.0,
                                                                    )),
                                                                    ..default()
                                                                },
                                                                BorderColor(Color::BLACK),
                                                                BackgroundColor(color.color()),
                                                            ))
                                                            .with_children(|button| {
                                                                button.spawn((
                                                                    Text::new(color.short_label()),
                                                                    TextFont::from_font_size(12.0),
                                                                    TextColor(Color::BLACK),
                                                                ));
                                                            });
                                                    }
                                                });
                                            });
                                    }
                                });

                            local
                                .spawn((
                                    Node {
                                        width: Val::Percent(38.0),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(10.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.86)),
                                ))
                                .with_children(|controls| {
                                    controls.spawn(menu_text("Global", 17.0));
                                    controls.spawn(menu_text("Players", 16.0));
                                    controls.spawn(Node::default()).with_children(|right| {
                                        spawn_player_row(right, &[2, 3, 6], menu.player_count);
                                    });
                                    controls.spawn(menu_text("AI Cooldown", 16.0));
                                    controls.spawn(Node::default()).with_children(|right| {
                                        spawn_ai_cooldown_row(right, menu.ai_cooldown_ms);
                                    });
                                });
                        });

                        step.spawn((NetworkOnly, menu_text("Role", 20.0)));
                        step.spawn((NetworkOnly, Node::default()))
                            .with_children(|network| {
                                spawn_network_mode_row(network, menu.net_mode);
                            });

                        step.spawn((NetworkOnly, menu_text("Server Address", 20.0)));
                        step.spawn((NetworkOnly, Node::default()))
                            .with_children(|network| {
                                network
                                    .spawn(button_bundle(
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
                                        NORMAL_BUTTON,
                                    ))
                                    .insert((
                                        NetworkAddressInputField,
                                        TextInput,
                                        TextInputValue(menu.net_address.clone()),
                                        TextInputInactive(true),
                                    ));
                            });

                        step.spawn((NetworkOnly, Node::default()))
                            .with_children(|network| {
                                network
                                    .spawn(button_bundle(
                                        NetworkConnectButton,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(40.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        NORMAL_BUTTON,
                                    ))
                                    .with_children(|button| {
                                        button.spawn(white_text("Apply Network Settings", 16.0));
                                    });
                            });

                        step.spawn((NetworkOnly, menu_text("Connected Players", 20.0)));
                        step.spawn((ConnectedPlayersText, NetworkOnly, menu_text("", 16.0)));

                        step.spawn(button_bundle(
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
                            MENU_START,
                        ))
                        .with_children(|button| {
                            button.spawn((StartGameButtonLabel, white_text("Start Game", 24.0)));
                        });
                    });

                panel
                    .spawn((
                        MenuScreenNetworkLobby,
                        Node {
                            width: Val::Percent(100.0),
                            row_gap: Val::Px(12.0),
                            flex_direction: FlexDirection::Column,
                            display: Display::None,
                            ..default()
                        },
                    ))
                    .with_children(|lobby| {
                        lobby
                            .spawn(button_bundle(
                                BackToModeButton,
                                button_node(120.0, 36.0, 1.0),
                                NORMAL_BUTTON,
                            ))
                            .with_children(|button| {
                                button.spawn(white_text("Back", 16.0));
                            });

                        lobby.spawn(menu_text("Network Lobby", 24.0));
                        lobby.spawn((ConnectedPlayersText, menu_text("", 16.0)));

                        lobby.spawn(menu_text("Pick Your Slot", 18.0));
                        lobby
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                row_gap: Val::Px(8.0),
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|slots| {
                                for slot in 0..6 {
                                    slots
                                        .spawn(button_bundle(
                                            NetworkSlotButton { slot },
                                            Node {
                                                width: Val::Px(84.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(1.0)),
                                                ..default()
                                            },
                                            NORMAL_BUTTON,
                                        ))
                                        .with_children(|button| {
                                            button
                                                .spawn(white_text(format!("P{}", slot + 1), 15.0));
                                        });
                                }
                            });

                        lobby.spawn((
                            NetworkLobbyHostOnly,
                            menu_text("Host controls game setup in the previous screen.", 14.0),
                        ));
                        lobby.spawn((
                            NetworkLobbyClientOnly,
                            menu_text("Waiting for host to start the match...", 14.0),
                        ));

                        lobby
                            .spawn((
                                StartGameButton,
                                NetworkLobbyHostOnly,
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
                                    white_text("Start Network Game", 24.0),
                                ));
                            });
                    });
            });

            root.spawn((
                AuthorsPopup,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.75)),
            ))
            .with_children(|overlay| {
                overlay
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            max_width: Val::Px(360.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            row_gap: Val::Px(10.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        BorderColor(Color::srgb(0.22, 0.22, 0.25)),
                        BackgroundColor(MENU_PANEL_BG),
                    ))
                    .with_children(|popup| {
                        popup.spawn(white_text("Authors", 24.0));
                        popup.spawn(menu_text("1. Codex", 17.0));
                        popup.spawn(menu_text("2. Janczar Knurek ;)", 17.0));
                        popup
                            .spawn(button_bundle(
                                MenuSettingsCloseButton,
                                button_node(120.0, 36.0, 1.0),
                                NORMAL_BUTTON,
                            ))
                            .with_children(|button| {
                                button.spawn(white_text("Close", 16.0));
                            });
                    });
            });

            root.spawn((
                RulesPopup,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.75)),
            ))
            .with_children(|overlay| {
                overlay
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            max_width: Val::Px(480.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            row_gap: Val::Px(10.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        BorderColor(Color::srgb(0.22, 0.22, 0.25)),
                        BackgroundColor(MENU_PANEL_BG),
                    ))
                    .with_children(|popup| {
                        popup.spawn(white_text("Rules", 24.0));
                        popup.spawn(menu_text("1. Reach your goal side to win.", 16.0));
                        popup.spawn(menu_text(
                            "2. On each turn, move one pawn OR place one fence.",
                            16.0,
                        ));
                        popup.spawn(menu_text(
                            "3. Fences block 3 edges and cannot trap any player.",
                            16.0,
                        ));
                        popup.spawn(menu_text(
                            "4. If blocked by a pawn, jump over it or sidestep when needed.",
                            16.0,
                        ));
                        popup
                            .spawn(button_bundle(
                                MenuSettingsCloseButton,
                                button_node(120.0, 36.0, 1.0),
                                NORMAL_BUTTON,
                            ))
                            .with_children(|button| {
                                button.spawn(white_text("Close", 16.0));
                            });
                    });
            });

            root.spawn((
                MenuSettingsPopup,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.75)),
            ))
            .with_children(|overlay| {
                overlay
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            max_width: Val::Px(420.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            row_gap: Val::Px(12.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            min_height: Val::Px(380.0),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.22, 0.22, 0.25)),
                        BackgroundColor(MENU_PANEL_BG),
                    ))
                    .with_children(|popup| {
                        popup.spawn(white_text("Settings", 24.0));

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
                                        width: Val::Px(120.0),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(TAB_ACTIVE),
                                ))
                                .with_children(|tab| {
                                    tab.spawn(white_text("Sound", 16.0));
                                });

                                tabs.spawn((
                                    Button,
                                    SettingsTabButton {
                                        tab: SettingsTab::Controls,
                                    },
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(TAB_INACTIVE),
                                ))
                                .with_children(|tab| {
                                    tab.spawn(white_text("Controls", 16.0));
                                });
                            });

                        popup
                            .spawn((
                                SettingsTabContent {
                                    tab: SettingsTab::Sound,
                                },
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(12.0),
                                    ..default()
                                },
                            ))
                            .with_children(|content| {
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

                        popup
                            .spawn((
                                SettingsTabContent {
                                    tab: SettingsTab::Controls,
                                },
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(10.0),
                                    display: Display::None,
                                    ..default()
                                },
                            ))
                            .with_children(|content| {
                                content
                                    .spawn(menu_text("Click a binding, then press a key.", 16.0));
                                spawn_control_binding_row(
                                    content,
                                    "Toggle Fence Mode",
                                    ControlBindingKind::ToggleFenceMode,
                                    app_settings.controls.toggle_fence_mode_label(),
                                );
                                spawn_control_binding_row(
                                    content,
                                    "Cycle Fence Shape",
                                    ControlBindingKind::CycleFenceShape,
                                    app_settings.controls.cycle_fence_shape_label(),
                                );
                                spawn_control_binding_row(
                                    content,
                                    "Rotate Fence",
                                    ControlBindingKind::RotateFenceOrientation,
                                    app_settings.controls.rotate_fence_orientation_label(),
                                );
                            });

                        popup
                            .spawn(button_bundle(
                                MenuSettingsCloseButton,
                                button_node(120.0, 36.0, 1.0),
                                NORMAL_BUTTON,
                            ))
                            .with_children(|button| {
                                button.spawn(white_text("Close", 16.0));
                            });
                    });
            });
        });
}

pub(super) fn handle_lobby_player_list_scroll(
    menu: Res<MenuSelection>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel_events: EventReader<MouseWheel>,
    mut lists: Query<(&RelativeCursorPosition, &mut ScrollPosition), With<LobbyPlayerListScroll>>,
) {
    if menu.screen != MenuScreen::Setup
        || (menu.game_mode != StartGameMode::Local
            && !(menu.game_mode == StartGameMode::Network
                && matches!(menu.net_mode, NetMode::Host)))
    {
        return;
    }

    let mut wheel_delta: f32 = 0.0;
    for event in wheel_events.read() {
        wheel_delta += event.y;
    }

    let mut key_delta: f32 = 0.0;
    if keys.just_pressed(KeyCode::ArrowDown) {
        key_delta += 56.0;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        key_delta -= 56.0;
    }
    if keys.just_pressed(KeyCode::PageDown) {
        key_delta += 220.0;
    }
    if keys.just_pressed(KeyCode::PageUp) {
        key_delta -= 220.0;
    }

    if wheel_delta.abs() <= f32::EPSILON && key_delta.abs() <= f32::EPSILON {
        return;
    }

    for (cursor, mut scroll) in &mut lists {
        let wheel_scroll = if cursor.normalized.is_some() {
            -wheel_delta * 36.0
        } else {
            0.0
        };
        let total_delta = wheel_scroll + key_delta;
        if total_delta.abs() <= f32::EPSILON {
            continue;
        }

        scroll.offset_y = (scroll.offset_y + total_delta).max(0.0);
    }
}

pub(super) fn cleanup_start_menu(
    mut commands: Commands,
    roots: Query<Entity, With<StartMenuRoot>>,
) {
    despawn_all!(commands, roots);
}

pub(super) fn handle_main_menu_action_buttons(
    mut menu: ResMut<MenuSelection>,
    mut settings_ui: ResMut<SettingsUiState>,
    mut exit_events: EventWriter<AppExit>,
    mut interactions: Query<
        (&Interaction, &MainMenuActionButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, button, mut color) in &mut interactions {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            match button.action {
                MainMenuAction::LocalGame => {
                    menu.game_mode = StartGameMode::Local;
                    menu.screen = MenuScreen::Setup;
                    settings_ui.pending_control_binding = None;
                }
                MainMenuAction::NetworkGame => {
                    menu.game_mode = StartGameMode::Network;
                    menu.screen = MenuScreen::Setup;
                    if matches!(menu.net_mode, NetMode::Local) {
                        menu.net_mode = NetMode::Host;
                    }
                    settings_ui.pending_control_binding = None;
                }
                MainMenuAction::Settings => {
                    menu.show_settings_popup = !menu.show_settings_popup;
                    if menu.show_settings_popup {
                        settings_ui.active_tab = SettingsTab::Sound;
                        settings_ui.pending_control_binding = None;
                    }
                }
                MainMenuAction::Rules => {
                    menu.show_rules_popup = !menu.show_rules_popup;
                    settings_ui.pending_control_binding = None;
                }
                MainMenuAction::Authors => {
                    menu.show_authors_popup = !menu.show_authors_popup;
                    settings_ui.pending_control_binding = None;
                }
                MainMenuAction::Quit => {
                    exit_events.write(AppExit::Success);
                }
            }
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn handle_back_to_mode_button(
    mut menu: ResMut<MenuSelection>,
    mut settings_ui: ResMut<SettingsUiState>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToModeButton>),
    >,
) {
    for (interaction, mut color) in &mut interactions {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            menu.screen = if menu.screen == MenuScreen::NetworkLobby {
                MenuScreen::Setup
            } else {
                MenuScreen::ModeSelect
            };
            menu.address_focused = false;
            menu.show_authors_popup = false;
            menu.show_rules_popup = false;
            menu.show_settings_popup = false;
            settings_ui.pending_control_binding = None;
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn handle_menu_settings_close_button(
    mut menu: ResMut<MenuSelection>,
    mut settings_ui: ResMut<SettingsUiState>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuSettingsCloseButton>),
    >,
) {
    for (interaction, mut color) in &mut interactions {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            menu.show_settings_popup = false;
            menu.show_authors_popup = false;
            menu.show_rules_popup = false;
            settings_ui.active_tab = SettingsTab::Sound;
            settings_ui.pending_control_binding = None;
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn handle_menu_settings_tab_buttons(
    menu: Res<MenuSelection>,
    mut settings_ui: ResMut<SettingsUiState>,
    mut tab_interactions: Query<
        (&Interaction, &SettingsTabButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !menu.show_settings_popup {
        return;
    }

    for (interaction, tab_button) in &mut tab_interactions {
        if *interaction == Interaction::Pressed {
            settings_ui.active_tab = tab_button.tab;
        }
    }
}

pub(super) fn handle_menu_control_binding_buttons(
    menu: Res<MenuSelection>,
    mut settings_ui: ResMut<SettingsUiState>,
    interactions: Query<
        (&Interaction, &ControlBindingButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !menu.show_settings_popup || settings_ui.active_tab != SettingsTab::Controls {
        return;
    }

    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            settings_ui.pending_control_binding = Some(button.kind);
        }
    }
}

pub(super) fn handle_menu_control_binding_capture(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<MenuSelection>,
    mut settings_ui: ResMut<SettingsUiState>,
    mut app_settings: ResMut<AppSettings>,
) {
    if !menu.show_settings_popup || settings_ui.active_tab != SettingsTab::Controls {
        return;
    }

    let Some(kind) = settings_ui.pending_control_binding else {
        return;
    };

    for key in keys.get_just_pressed() {
        let changed = apply_control_binding(&mut app_settings, kind, *key);
        settings_ui.pending_control_binding = None;
        if changed {
            let _ = settings::save_settings_to_disk(app_settings.clone());
        }
        break;
    }
}

pub(super) fn sync_menu_control_binding_texts(
    app_settings: Res<AppSettings>,
    settings_ui: Res<SettingsUiState>,
    mut texts: Query<(&ControlBindingValueText, &mut Text)>,
) {
    if !app_settings.is_changed() && !settings_ui.is_changed() {
        return;
    }

    for (value_text, mut text) in &mut texts {
        if settings_ui.pending_control_binding == Some(value_text.kind)
            && settings_ui.active_tab == SettingsTab::Controls
        {
            *text = Text::new("Press key...");
        } else {
            *text = Text::new(control_binding_label(&app_settings, value_text.kind));
        }
    }
}

pub(super) fn sync_menu_settings_tab_visibility(
    settings_ui: Res<SettingsUiState>,
    mut tab_button_query: Query<(&SettingsTabButton, &mut BackgroundColor), With<Button>>,
    mut tab_content_query: Query<(&SettingsTabContent, &mut Node)>,
) {
    if !settings_ui.is_changed() {
        return;
    }

    for (tab_button, mut tab_color) in &mut tab_button_query {
        *tab_color = if tab_button.tab == settings_ui.active_tab {
            TAB_ACTIVE.into()
        } else {
            TAB_INACTIVE.into()
        };
    }

    for (tab_content, mut node) in &mut tab_content_query {
        node.display = if tab_content.tab == settings_ui.active_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn handle_menu_sound_slider_input(
    mut app_settings: ResMut<AppSettings>,
    track_query: Query<
        (
            &Interaction,
            &bevy::ui::RelativeCursorPosition,
            &SoundSliderTrack,
        ),
        With<Button>,
    >,
) {
    let mut changed = false;
    for (interaction, cursor_pos, slider) in &track_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(normalized) = cursor_pos.normalized else {
            continue;
        };
        slider.kind.set_value(&mut app_settings, normalized.x);
        changed = true;
    }

    if changed {
        let _ = settings::save_settings_to_disk(app_settings.clone());
    }
}

pub(super) fn sync_menu_sound_slider_visuals(
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

pub(super) fn handle_menu_option_buttons(
    mut menu: ResMut<MenuSelection>,
    net_config: Res<NetConfig>,
    net_lobby: Res<NetLobbyState>,
    mut net_ui_commands: EventWriter<NetUiCommand>,
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
    mut ai_cooldown_buttons: Query<
        (&Interaction, &AiCooldownButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<AiCooldownButton>,
            Without<BoardSizeButton>,
            Without<PlayerCountButton>,
        ),
    >,
    mut player_control_toggle_buttons: Query<
        (&Interaction, &PlayerControlToggleButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<PlayerControlToggleButton>,
        ),
    >,
    mut player_detail_dropdown_buttons: Query<
        (&Interaction, &PlayerDetailDropdownButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<PlayerDetailDropdownButton>,
        ),
    >,
    mut player_detail_option_buttons: Query<
        (&Interaction, &PlayerDetailOptionButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<PlayerDetailOptionButton>,
        ),
    >,
    mut player_color_buttons: Query<
        (&Interaction, &PlayerColorButton),
        (Changed<Interaction>, With<Button>, With<PlayerColorButton>),
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
    if menu.screen != MenuScreen::Setup && menu.screen != MenuScreen::NetworkLobby {
        return;
    }

    let mut lobby_dirty = false;
    let mut client_slot_for_sync = net_lobby.client_slot;
    let can_edit_full_lobby = menu.game_mode == StartGameMode::Local
        || (menu.game_mode == StartGameMode::Network && matches!(net_config.mode, NetMode::Host));

    for (interaction, button) in &mut board_buttons {
        if *interaction == Interaction::Pressed && can_edit_full_lobby {
            menu.board_radius = button.radius;
            lobby_dirty = true;
        }
    }

    for (interaction, button) in &mut player_buttons {
        if *interaction == Interaction::Pressed && can_edit_full_lobby {
            menu.player_count = button.player_count;
            lobby_dirty = true;
        }
    }

    for (interaction, button) in &mut ai_cooldown_buttons {
        if *interaction == Interaction::Pressed && can_edit_full_lobby {
            menu.ai_cooldown_ms = button.cooldown_ms;
            lobby_dirty = true;
        }
    }

    for (interaction, button) in &mut player_control_toggle_buttons {
        if *interaction == Interaction::Pressed && can_edit_full_lobby {
            let current = menu.player_controls[button.player_index];
            let toggled = if current.is_ai() {
                PlayerControl::Human
            } else {
                PlayerControl::RandomAi
            };
            menu.player_controls[button.player_index] = toggled;
            if matches!(net_config.mode, NetMode::Host) && toggled.is_ai() {
                if menu.network_local_slot == Some(button.player_index) {
                    menu.network_local_slot = None;
                }
                if client_slot_for_sync == Some(button.player_index) {
                    client_slot_for_sync = None;
                }
            }
            menu.open_player_detail_dropdown = None;
            lobby_dirty = true;
        }
    }

    for (interaction, button) in &mut player_detail_dropdown_buttons {
        if *interaction == Interaction::Pressed {
            if !player_detail_dropdown_enabled(&menu, &net_config, button.player_index) {
                continue;
            }
            menu.open_player_detail_dropdown =
                if menu.open_player_detail_dropdown == Some(button.player_index) {
                    None
                } else {
                    Some(button.player_index)
                };
        }
    }

    for (interaction, button) in &mut player_color_buttons {
        if *interaction == Interaction::Pressed && can_edit_full_lobby {
            if let Some(other_player_index) = (0..menu.player_count).find(|player_index| {
                *player_index != button.player_index
                    && menu.player_colors[*player_index] == button.color
            }) {
                menu.player_colors
                    .swap(button.player_index, other_player_index);
            } else {
                menu.player_colors[button.player_index] = button.color;
            }
            lobby_dirty = true;
        }
    }

    for (interaction, button) in &mut network_mode_buttons {
        if *interaction == Interaction::Pressed {
            menu.net_mode = button.mode;
        }
    }

    for (interaction, button) in &mut player_detail_option_buttons {
        if *interaction != Interaction::Pressed
            || button.player_index >= menu.player_count
            || !player_detail_option_enabled(&menu, &net_config, button.player_index, button.option)
        {
            continue;
        }

        menu.open_player_detail_dropdown = None;
        match (net_config.mode, button.option) {
            (NetMode::Host, PlayerDetailOption::Host) => {
                menu.player_controls[button.player_index] = PlayerControl::Human;
                menu.network_local_slot = Some(button.player_index);
                if client_slot_for_sync == Some(button.player_index) {
                    client_slot_for_sync = None;
                }
                lobby_dirty = true;
            }
            (NetMode::Host, PlayerDetailOption::Client) => {
                menu.player_controls[button.player_index] = PlayerControl::Human;
                if menu.network_local_slot == Some(button.player_index) {
                    menu.network_local_slot = None;
                }
                client_slot_for_sync = Some(button.player_index);
                lobby_dirty = true;
            }
            (NetMode::Client, PlayerDetailOption::Client) => {
                net_ui_commands.write(NetUiCommand::SelectLocalSlot(Some(button.player_index)));
            }
            (_, PlayerDetailOption::Heuristic) if can_edit_full_lobby => {
                menu.player_ai_strategies[button.player_index] = AiStrategy::Heuristic;
                lobby_dirty = true;
            }
            (_, PlayerDetailOption::AlphaBeta) if can_edit_full_lobby => {
                menu.player_ai_strategies[button.player_index] = AiStrategy::AlphaBeta;
                lobby_dirty = true;
            }
            _ => {}
        }
    }

    if lobby_dirty
        && matches!(menu.game_mode, StartGameMode::Network)
        && matches!(net_config.mode, NetMode::Host)
    {
        let mut synced = GameConfig::default();
        synced.board_radius = menu.board_radius;
        synced.player_count = menu.player_count;
        synced.player_controls = menu.player_controls;
        synced.player_ai_strategies = menu.player_ai_strategies;
        synced.player_colors = menu.player_colors;
        synced.ai_cooldown_seconds = menu.ai_cooldown_ms as f32 / 1_000.0;
        net_ui_commands.write(NetUiCommand::HostSyncLobby {
            config: synced,
            host_slot: menu.network_local_slot,
            client_slot: client_slot_for_sync,
        });
    }
}

pub(super) fn handle_network_connect_button(
    mut menu: ResMut<MenuSelection>,
    mut net_config: ResMut<NetConfig>,
    net_lobby: Res<NetLobbyState>,
    mut net_ui_commands: EventWriter<NetUiCommand>,
    mut app_settings: ResMut<AppSettings>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<NetworkConnectButton>),
    >,
) {
    if menu.screen != MenuScreen::Setup || menu.game_mode != StartGameMode::Network {
        return;
    }

    for (interaction, mut color) in &mut interactions {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            net_config.mode = menu.net_mode;
            net_config.local_player_index = local_player_index_for_mode(menu.net_mode);
            let trimmed = menu.net_address.trim();
            net_config.address = if trimmed.is_empty() {
                "127.0.0.1:4000".to_string()
            } else {
                trimmed.to_string()
            };
            save_last_network_settings(&mut app_settings, menu.net_mode, &net_config.address);
            if matches!(menu.net_mode, NetMode::Host) {
                menu.network_local_slot = Some(0);
                let mut synced = GameConfig::default();
                synced.board_radius = menu.board_radius;
                synced.player_count = menu.player_count;
                synced.player_controls = menu.player_controls;
                synced.player_ai_strategies = menu.player_ai_strategies;
                synced.player_colors = menu.player_colors;
                synced.ai_cooldown_seconds = menu.ai_cooldown_ms as f32 / 1_000.0;
                net_ui_commands.write(NetUiCommand::HostSyncLobby {
                    config: synced,
                    host_slot: menu.network_local_slot,
                    client_slot: net_lobby.client_slot,
                });
            }
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn handle_network_address_focus(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut menu: ResMut<MenuSelection>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor, &mut TextInputInactive),
        With<NetworkAddressInputButton>,
    >,
) {
    if menu.screen != MenuScreen::Setup || menu.game_mode != StartGameMode::Network {
        return;
    }

    for (interaction, mut color, mut inactive) in &mut interactions {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            let focused = *interaction == Interaction::Pressed;
            inactive.0 = !focused;
            menu.address_focused = focused;
        }

        *color = if menu.address_focused {
            MENU_SELECTED.into()
        } else {
            neutral_button_color(*interaction).into()
        }
    }
}

pub(super) fn sync_network_lobby_screen(
    net_config: Res<NetConfig>,
    net_runtime: Res<NetRuntime>,
    net_lobby: Res<NetLobbyState>,
    mut menu: ResMut<MenuSelection>,
) {
    if menu.game_mode != StartGameMode::Network {
        return;
    }

    if menu.screen == MenuScreen::Setup
        && menu.game_mode == StartGameMode::Network
        && net_runtime.connected
    {
        menu.sync_from_network_lobby(net_config.mode, &net_lobby);
    }
}

pub(super) fn handle_network_address_typing(
    mut menu: ResMut<MenuSelection>,
    mut input_query: Query<
        (&mut TextInputValue, &TextInputInactive),
        With<NetworkAddressInputField>,
    >,
) {
    if menu.screen != MenuScreen::Setup || menu.game_mode != StartGameMode::Network {
        return;
    }

    let Ok((mut value, inactive)) = input_query.single_mut() else {
        return;
    };

    let is_active = !inactive.0;
    if menu.address_focused != is_active {
        menu.address_focused = is_active;
    }

    let sanitized = sanitize_address(value.0.as_str());
    if value.0 != sanitized {
        value.0 = sanitized.clone();
    }
    if menu.net_address != sanitized {
        menu.net_address = sanitized;
    }
}

pub(super) fn sync_network_address_input_from_menu(
    menu: Res<MenuSelection>,
    mut input_query: Query<&mut TextInputValue, With<NetworkAddressInputField>>,
) {
    if !menu.is_changed() {
        return;
    }

    if let Ok(mut value) = input_query.single_mut()
        && value.0 != menu.net_address
    {
        value.0 = menu.net_address.clone();
    }
}

pub(super) fn sync_menu_layout_visibility(
    menu: Res<MenuSelection>,
    net_config: Res<NetConfig>,
    mut sections: Query<(
        Option<&MenuScreenModeSelect>,
        Option<&MenuScreenSetup>,
        Option<&MenuScreenNetworkLobby>,
        Option<&AuthorsPopup>,
        Option<&RulesPopup>,
        Option<&MenuSettingsPopup>,
        Option<&LocalOnly>,
        Option<&NetworkOnly>,
        Option<&PlayerSetupRow>,
        Option<&PlayerAiOnly>,
        Option<&PlayerDetailDropdownMenu>,
        Option<&PlayerDetailOptionButton>,
        Option<&NetworkLobbyHostOnly>,
        Option<&NetworkLobbyClientOnly>,
        &mut Node,
    )>,
) {
    if !menu.is_changed() {
        return;
    }

    for (
        mode_screen,
        setup_screen,
        network_lobby_screen,
        authors_popup,
        rules_popup,
        settings_popup,
        local_only,
        network_only,
        player_row,
        ai_only,
        detail_menu,
        detail_option,
        network_lobby_host_only,
        network_lobby_client_only,
        mut node,
    ) in &mut sections
    {
        if mode_screen.is_some() {
            node.display = if menu.screen == MenuScreen::ModeSelect
                && !menu.show_authors_popup
                && !menu.show_rules_popup
                && !menu.show_settings_popup
            {
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
        } else if network_lobby_screen.is_some() {
            node.display = if menu.screen == MenuScreen::NetworkLobby {
                Display::Flex
            } else {
                Display::None
            };
        } else if authors_popup.is_some() {
            node.display = if menu.screen == MenuScreen::ModeSelect && menu.show_authors_popup {
                Display::Flex
            } else {
                Display::None
            };
        } else if rules_popup.is_some() {
            node.display = if menu.screen == MenuScreen::ModeSelect && menu.show_rules_popup {
                Display::Flex
            } else {
                Display::None
            };
        } else if settings_popup.is_some() {
            node.display = if menu.screen == MenuScreen::ModeSelect && menu.show_settings_popup {
                Display::Flex
            } else {
                Display::None
            };
        } else if local_only.is_some() {
            node.display = if menu.screen == MenuScreen::Setup
                && (menu.game_mode == StartGameMode::Local
                    || menu.game_mode == StartGameMode::Network)
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
        } else if let Some(row) = player_row {
            node.display = if menu.screen == MenuScreen::Setup
                && (menu.game_mode == StartGameMode::Local
                    || menu.game_mode == StartGameMode::Network)
                && row.player_index < menu.player_count
            {
                Display::Flex
            } else {
                Display::None
            };
        } else if let Some(ai_row) = ai_only {
            node.display = if menu.screen == MenuScreen::Setup
                && (menu.game_mode == StartGameMode::Local
                    || menu.game_mode == StartGameMode::Network)
                && ai_row.player_index < menu.player_count
                && menu.player_controls[ai_row.player_index].is_ai()
            {
                Display::Flex
            } else {
                Display::None
            };
        } else if let Some(detail_menu) = detail_menu {
            node.display = if menu.open_player_detail_dropdown == Some(detail_menu.player_index)
                && menu.screen == MenuScreen::Setup
                && detail_menu.player_index < menu.player_count
                && player_detail_dropdown_enabled(&menu, &net_config, detail_menu.player_index)
            {
                Display::Flex
            } else {
                Display::None
            };
        } else if let Some(detail_option) = detail_option {
            node.display = if menu.open_player_detail_dropdown == Some(detail_option.player_index)
                && menu.screen == MenuScreen::Setup
                && detail_option.player_index < menu.player_count
                && player_detail_option_enabled(
                    &menu,
                    &net_config,
                    detail_option.player_index,
                    detail_option.option,
                ) {
                Display::Flex
            } else {
                Display::None
            };
        } else if network_lobby_host_only.is_some() {
            node.display = if menu.screen == MenuScreen::NetworkLobby
                && matches!(net_config.mode, NetMode::Host)
            {
                Display::Flex
            } else {
                Display::None
            };
        } else if network_lobby_client_only.is_some() {
            node.display = if menu.screen == MenuScreen::NetworkLobby
                && matches!(net_config.mode, NetMode::Client)
            {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

pub(super) fn sync_menu_main_panel_width(
    menu: Res<MenuSelection>,
    mut panels: Query<&mut Node, With<MenuMainPanel>>,
) {
    if !menu.is_changed() {
        return;
    }

    let show_wide_setup = menu.screen == MenuScreen::Setup
        && (menu.game_mode == StartGameMode::Local || menu.game_mode == StartGameMode::Network);

    let width = if show_wide_setup {
        Val::Px(980.0)
    } else {
        Val::Px(480.0)
    };
    let max_width = if show_wide_setup {
        Val::Percent(96.0)
    } else {
        Val::Percent(92.0)
    };

    for mut panel in &mut panels {
        panel.width = width;
        panel.max_width = max_width;
    }
}

pub(super) fn sync_menu_button_visuals(
    menu: Res<MenuSelection>,
    net_config: Res<NetConfig>,
    net_lobby: Res<NetLobbyState>,
    settings_ui: Res<SettingsUiState>,
    net_runtime: Res<NetRuntime>,
    mut option_buttons: Query<
        (
            &Interaction,
            Option<&BoardSizeButton>,
            Option<&PlayerCountButton>,
            Option<&AiCooldownButton>,
            Option<&PlayerControlToggleButton>,
            Option<&PlayerDetailDropdownButton>,
            Option<&PlayerDetailOptionButton>,
            Option<&PlayerColorButton>,
            Option<&MainMenuActionButton>,
            Option<&NetworkModeButton>,
            Option<&NetworkAddressInputButton>,
            Option<&ControlBindingButton>,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
    mut menu_texts: Query<(
        Option<&ConnectedPlayersText>,
        Option<&StartGameButtonLabel>,
        &mut Text,
    )>,
) {
    for (
        interaction,
        board,
        player,
        ai_cooldown,
        player_toggle,
        player_detail_dropdown,
        player_detail_option,
        player_color,
        menu_action,
        network_mode,
        address_input,
        control_binding,
        mut color,
    ) in &mut option_buttons
    {
        *color = if let Some(button) = board {
            selected_button_color(button.radius == menu.board_radius, *interaction).into()
        } else if let Some(button) = player {
            selected_button_color(button.player_count == menu.player_count, *interaction).into()
        } else if let Some(button) = ai_cooldown {
            selected_button_color(button.cooldown_ms == menu.ai_cooldown_ms, *interaction).into()
        } else if let Some(button) = player_toggle {
            selected_button_color(
                menu.player_controls[button.player_index].is_ai(),
                *interaction,
            )
            .into()
        } else if let Some(button) = player_detail_dropdown {
            selected_button_color(
                menu.open_player_detail_dropdown == Some(button.player_index),
                *interaction,
            )
            .into()
        } else if let Some(button) = player_detail_option {
            selected_button_color(
                player_detail_option_enabled(
                    &menu,
                    &net_config,
                    button.player_index,
                    button.option,
                ) && selected_player_detail_option(&menu, &net_lobby, button.player_index)
                    == Some(button.option),
                *interaction,
            )
            .into()
        } else if let Some(button) = player_color {
            player_color_button_color(
                button.color,
                menu.player_colors[button.player_index] == button.color,
                *interaction,
            )
            .into()
        } else if menu_action.is_some() {
            neutral_button_color(*interaction).into()
        } else if let Some(button) = network_mode {
            selected_button_color(button.mode == menu.net_mode, *interaction).into()
        } else if address_input.is_some() {
            selected_button_color(menu.address_focused, *interaction).into()
        } else if let Some(binding) = control_binding {
            selected_button_color(
                settings_ui.pending_control_binding == Some(binding.kind),
                *interaction,
            )
            .into()
        } else {
            *color
        };
    }

    for (connected_text, start_text, mut text) in &mut menu_texts {
        if connected_text.is_some() {
            *text = Text::new(connected_players_label(
                menu.game_mode,
                menu.net_mode,
                net_runtime.connected,
                &net_lobby,
            ));
        } else if start_text.is_some() {
            let label = if menu.screen == MenuScreen::NetworkLobby {
                if matches!(net_config.mode, NetMode::Client) {
                    "Waiting for Host"
                } else {
                    "Start Network Game"
                }
            } else if menu.game_mode == StartGameMode::Network && menu.net_mode == NetMode::Client {
                "Waiting for Host"
            } else {
                "Start Game"
            };
            *text = Text::new(label);
        }
    }
}

pub(super) fn sync_player_detail_labels(
    menu: Res<MenuSelection>,
    net_lobby: Res<NetLobbyState>,
    mut labels: ParamSet<(
        Query<(&PlayerControlButton, &mut Text)>,
        Query<(&PlayerDetailDropdownText, &mut Text)>,
    )>,
) {
    if !menu.is_changed() && !net_lobby.is_changed() {
        return;
    }

    for (label, mut text) in &mut labels.p0() {
        let value = if menu.player_controls[label.player_index].is_ai() {
            "AI"
        } else {
            "Human"
        };
        *text = Text::new(value);
    }

    for (label, mut text) in &mut labels.p1() {
        let value = player_detail_label(&menu, &net_lobby, label.player_index);
        *text = Text::new(value);
    }
}

pub(super) fn touch_legacy_network_slot_buttons(buttons: Query<&NetworkSlotButton, With<Button>>) {
    for button in &buttons {
        let _ = button.slot;
    }
}

fn apply_control_binding(
    app_settings: &mut AppSettings,
    binding_kind: ControlBindingKind,
    key: KeyCode,
) -> bool {
    match binding_kind {
        ControlBindingKind::ToggleFenceMode => app_settings.controls.set_toggle_fence_mode_key(key),
        ControlBindingKind::CycleFenceShape => app_settings.controls.set_cycle_fence_shape_key(key),
        ControlBindingKind::RotateFenceOrientation => {
            app_settings.controls.set_rotate_fence_orientation_key(key)
        }
    }
}

fn control_binding_label(
    app_settings: &AppSettings,
    binding_kind: ControlBindingKind,
) -> &'static str {
    match binding_kind {
        ControlBindingKind::ToggleFenceMode => app_settings.controls.toggle_fence_mode_label(),
        ControlBindingKind::CycleFenceShape => app_settings.controls.cycle_fence_shape_label(),
        ControlBindingKind::RotateFenceOrientation => {
            app_settings.controls.rotate_fence_orientation_label()
        }
    }
}

pub(super) fn handle_start_game_button(
    menu: Res<MenuSelection>,
    mut net_config: ResMut<NetConfig>,
    mut game_config: ResMut<GameConfig>,
    net_lobby: Res<NetLobbyState>,
    mut app_settings: ResMut<AppSettings>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartGameButton>),
    >,
    mut next_phase: ResMut<NextState<AppPhase>>,
) {
    if menu.screen != MenuScreen::Setup && menu.screen != MenuScreen::NetworkLobby {
        return;
    }

    for (interaction, mut color) in &mut interactions {
        if menu.game_mode == StartGameMode::Network && matches!(net_config.mode, NetMode::Client) {
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
                net_config.local_player_index = if menu.game_mode == StartGameMode::Network {
                    menu.network_local_slot.unwrap_or(0)
                } else {
                    local_player_index_for_mode(net_config.mode)
                };
                let trimmed = menu.net_address.trim();
                net_config.address = if trimmed.is_empty() {
                    "127.0.0.1:4000".to_string()
                } else {
                    trimmed.to_string()
                };
                if menu.game_mode == StartGameMode::Network {
                    save_last_network_settings(
                        &mut app_settings,
                        menu.net_mode,
                        &net_config.address,
                    );
                }
                game_config.board_radius = menu.board_radius;
                if menu.game_mode == StartGameMode::Local {
                    game_config.player_count = menu.player_count;
                    game_config.ai_cooldown_seconds = menu.ai_cooldown_ms as f32 / 1_000.0;
                    game_config.player_controls = [PlayerControl::Human; 6];
                    game_config.player_ai_strategies = [AiStrategy::Heuristic; 6];
                    game_config.player_colors = menu.player_colors;
                    for player_index in 0..menu.player_count {
                        game_config.player_controls[player_index] =
                            menu.player_controls[player_index];
                        game_config.player_ai_strategies[player_index] =
                            menu.player_ai_strategies[player_index];
                    }
                } else {
                    let host_slot = menu.network_local_slot.or(net_lobby.host_slot).unwrap_or(0);
                    let client_slot = net_lobby.client_slot;
                    if host_slot >= menu.player_count {
                        *color = NORMAL_BUTTON.into();
                        continue;
                    }
                    *game_config = build_network_game_config(&menu, host_slot, client_slot);
                    net_config.local_player_index = host_slot;
                }
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

fn connected_players_label(
    game_mode: StartGameMode,
    net_mode: NetMode,
    connected: bool,
    net_lobby: &NetLobbyState,
) -> String {
    if game_mode != StartGameMode::Network {
        return "Not used in local mode.".to_string();
    }

    let host_slot = net_lobby
        .host_slot
        .map(|slot| format!("P{}", slot + 1))
        .unwrap_or_else(|| "none".to_string());
    let client_slot = net_lobby
        .client_slot
        .map(|slot| format!("P{}", slot + 1))
        .unwrap_or_else(|| "none".to_string());

    match net_mode {
        NetMode::Host => format!(
            "Connection: {}\nHost slot: {}\nClient slot: {}",
            if connected {
                "Client connected"
            } else {
                "Waiting for client"
            },
            host_slot,
            client_slot
        ),
        NetMode::Client => format!(
            "Connection: {}\nHost slot: {}\nYour slot: {}",
            if connected {
                "Connected"
            } else {
                "Connecting..."
            },
            host_slot,
            client_slot
        ),
        NetMode::Local => "Choose Host or Client.".to_string(),
    }
}

fn player_detail_label(
    menu: &MenuSelection,
    net_lobby: &NetLobbyState,
    player_index: usize,
) -> &'static str {
    if menu.player_controls[player_index].is_ai() {
        match menu.player_ai_strategies[player_index] {
            AiStrategy::Heuristic => "Heuristic",
            AiStrategy::AlphaBeta => "AlphaBeta",
        }
    } else if menu.game_mode == StartGameMode::Network {
        match network_slot_owner_for_player(player_index, menu, net_lobby) {
            NetworkSlotOwner::Host => "Host",
            NetworkSlotOwner::Client => "Client",
            NetworkSlotOwner::Ai => "Unassigned",
        }
    } else {
        "Local Player"
    }
}

fn selected_player_detail_option(
    menu: &MenuSelection,
    net_lobby: &NetLobbyState,
    player_index: usize,
) -> Option<PlayerDetailOption> {
    if menu.player_controls[player_index].is_ai() {
        return Some(match menu.player_ai_strategies[player_index] {
            AiStrategy::Heuristic => PlayerDetailOption::Heuristic,
            AiStrategy::AlphaBeta => PlayerDetailOption::AlphaBeta,
        });
    }
    if menu.game_mode == StartGameMode::Network {
        return Some(
            match network_slot_owner_for_player(player_index, menu, net_lobby) {
                NetworkSlotOwner::Host => PlayerDetailOption::Host,
                NetworkSlotOwner::Client => PlayerDetailOption::Client,
                NetworkSlotOwner::Ai => return None,
            },
        );
    }
    None
}

fn player_detail_dropdown_enabled(
    menu: &MenuSelection,
    net_config: &NetConfig,
    player_index: usize,
) -> bool {
    if player_index >= menu.player_count {
        return false;
    }
    if menu.player_controls[player_index].is_ai() {
        return menu.game_mode == StartGameMode::Local
            || (menu.game_mode == StartGameMode::Network
                && matches!(net_config.mode, NetMode::Host));
    }
    menu.game_mode == StartGameMode::Network
}

fn player_detail_option_enabled(
    menu: &MenuSelection,
    net_config: &NetConfig,
    player_index: usize,
    option: PlayerDetailOption,
) -> bool {
    if player_index >= menu.player_count {
        return false;
    }

    if menu.player_controls[player_index].is_ai() {
        if !(menu.game_mode == StartGameMode::Local
            || (menu.game_mode == StartGameMode::Network
                && matches!(net_config.mode, NetMode::Host)))
        {
            return false;
        }
        return matches!(
            option,
            PlayerDetailOption::Heuristic | PlayerDetailOption::AlphaBeta
        );
    }

    if menu.game_mode != StartGameMode::Network {
        return false;
    }

    match net_config.mode {
        NetMode::Host => matches!(
            option,
            PlayerDetailOption::Host | PlayerDetailOption::Client
        ),
        NetMode::Client => matches!(option, PlayerDetailOption::Client),
        NetMode::Local => false,
    }
}

fn network_slot_owner_for_player(
    player_index: usize,
    menu: &MenuSelection,
    net_lobby: &NetLobbyState,
) -> NetworkSlotOwner {
    let host_slot = menu.network_local_slot.or(net_lobby.host_slot);
    if host_slot == Some(player_index) {
        return NetworkSlotOwner::Host;
    }
    if net_lobby.client_slot == Some(player_index) {
        return NetworkSlotOwner::Client;
    }
    NetworkSlotOwner::Ai
}

fn player_color_button_color(
    color: PlayerColor,
    selected: bool,
    interaction: Interaction,
) -> Color {
    if selected {
        return color.color();
    }

    let base = color.color().to_srgba();
    match interaction {
        Interaction::Hovered => Color::srgba(base.red, base.green, base.blue, 0.85),
        Interaction::Pressed => color.color(),
        Interaction::None => Color::srgba(base.red, base.green, base.blue, 0.45),
    }
}

fn build_network_game_config(
    menu: &MenuSelection,
    host_slot: usize,
    client_slot: Option<usize>,
) -> GameConfig {
    let mut config = GameConfig::default();
    config.board_radius = menu.board_radius;
    config.player_count = menu.player_count;
    config.ai_cooldown_seconds = menu.ai_cooldown_ms as f32 / 1_000.0;
    config.player_controls = [PlayerControl::RandomAi; 6];
    config.player_ai_strategies = menu.player_ai_strategies;
    config.player_colors = menu.player_colors;

    for player_index in 0..config.player_count {
        if player_index == host_slot || client_slot == Some(player_index) {
            config.player_controls[player_index] = PlayerControl::Human;
        } else {
            config.player_controls[player_index] = menu.player_controls[player_index];
        }
    }

    config
}

fn is_valid_address_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '-')
}

fn sanitize_address(raw: &str) -> String {
    let filtered: String = raw
        .chars()
        .filter(|ch| is_valid_address_char(*ch))
        .collect();
    let trimmed = filtered.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.chars().take(80).collect()
    }
}

fn nearest_ai_cooldown_ms(cooldown_seconds: f32) -> u32 {
    let target_ms = (cooldown_seconds.max(0.0) * 1_000.0).round() as i64;
    let mut best = AI_COOLDOWN_CHOICES_MS[0];
    let mut best_distance = (best as i64 - target_ms).abs();

    for choice in AI_COOLDOWN_CHOICES_MS {
        let distance = (choice as i64 - target_ms).abs();
        if distance < best_distance {
            best = choice;
            best_distance = distance;
        }
    }

    best
}

fn save_last_network_settings(app_settings: &mut AppSettings, mode: NetMode, address: &str) {
    let mapped_mode = match mode {
        NetMode::Host => LastNetMode::Host,
        NetMode::Client => LastNetMode::Client,
        NetMode::Local => return,
    };

    app_settings.network.mode = mapped_mode;
    app_settings.network.address = address.to_string();
    app_settings.network.local_player_index = local_player_index_for_mode(mode);
    let _ = settings::save_settings_to_disk(app_settings.clone());
}
