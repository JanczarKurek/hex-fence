use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::app_state::{AiStrategy, AppPhase, GameConfig, PlayerControl};
use crate::network::{NetConfig, NetMode, NetRuntime};
use crate::settings::{self, AppSettings, LastNetMode};

use super::components::{
    AiCooldownButton, AiPlayerCountButton, AiStrategyButton, BackToModeButton, BoardSizeButton,
    ConnectedPlayersText, LocalOnly, MenuScreen, MenuScreenModeSelect, MenuScreenSetup,
    MenuSelection, ModeChoiceButton, NetworkAddressInputButton, NetworkAddressText,
    NetworkConnectButton, NetworkModeButton, NetworkOnly, PlayerCountButton, StartGameButton,
    StartGameButtonLabel, StartGameMode, StartMenuRoot,
};
use super::styles::{
    HOVERED_BUTTON, MENU_PANEL_BG, MENU_SELECTED, MENU_START, NORMAL_BUTTON, PRESSED_BUTTON,
    button_bundle, button_node, menu_text, neutral_button_color, row_node, selected_button_color,
    white_text,
};
use super::widgets::{
    spawn_ai_cooldown_row, spawn_ai_player_row, spawn_ai_strategy_row, spawn_choice_row,
    spawn_network_mode_row, spawn_player_row,
};

const AI_COOLDOWN_CHOICES_MS: [u32; 5] = [250, 500, 1_000, 1_500, 2_000];

pub(super) fn setup_start_menu(
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
    menu.ai_player_count = game_config
        .player_controls
        .iter()
        .take(game_config.player_count)
        .filter(|control| control.is_ai())
        .count();
    menu.ai_cooldown_ms = nearest_ai_cooldown_ms(game_config.ai_cooldown_seconds);
    menu.ai_strategy = game_config.ai_strategy;
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
                panel.spawn(white_text("Hex Board Setup", 34.0));

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
                        step.spawn(menu_text("Choose Mode", 20.0));
                        step.spawn(row_node(10.0)).with_children(|row| {
                            row.spawn(button_bundle(
                                ModeChoiceButton {
                                    mode: StartGameMode::Local,
                                },
                                button_node(220.0, 44.0, 1.0),
                                NORMAL_BUTTON,
                            ))
                            .with_children(|button| {
                                button.spawn(white_text("Local Multiplayer", 18.0));
                            });
                            row.spawn(button_bundle(
                                ModeChoiceButton {
                                    mode: StartGameMode::Network,
                                },
                                button_node(240.0, 44.0, 1.0),
                                NORMAL_BUTTON,
                            ))
                            .with_children(|button| {
                                button.spawn(white_text("Network Multiplayer", 18.0));
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

                        step.spawn((LocalOnly, menu_text("Players", 20.0)));
                        step.spawn((LocalOnly, Node::default()))
                            .with_children(|local| {
                                spawn_player_row(local, &[2, 3, 6], menu.player_count);
                            });
                        step.spawn((LocalOnly, menu_text("AI Players", 20.0)));
                        step.spawn((LocalOnly, Node::default()))
                            .with_children(|local| {
                                spawn_ai_player_row(local, menu.ai_player_count);
                            });
                        step.spawn((LocalOnly, menu_text("AI Cooldown", 20.0)));
                        step.spawn((LocalOnly, Node::default()))
                            .with_children(|local| {
                                spawn_ai_cooldown_row(local, menu.ai_cooldown_ms);
                            });
                        step.spawn((LocalOnly, menu_text("AI Type", 20.0)));
                        step.spawn((LocalOnly, Node::default()))
                            .with_children(|local| {
                                spawn_ai_strategy_row(local, menu.ai_strategy);
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
                                    .with_children(|input| {
                                        input.spawn((
                                            NetworkAddressText,
                                            white_text(menu.net_address.clone(), 16.0),
                                        ));
                                    });
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
            });
        });
}

pub(super) fn cleanup_start_menu(
    mut commands: Commands,
    roots: Query<Entity, With<StartMenuRoot>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

pub(super) fn handle_mode_choice_buttons(
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

pub(super) fn handle_back_to_mode_button(
    mut menu: ResMut<MenuSelection>,
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<BackToModeButton>),
    >,
) {
    for (interaction, mut color) in &mut interactions {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            menu.screen = MenuScreen::ModeSelect;
            menu.address_focused = false;
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn handle_menu_option_buttons(
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
    mut ai_player_buttons: Query<
        (&Interaction, &AiPlayerCountButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<AiPlayerCountButton>,
            Without<BoardSizeButton>,
            Without<PlayerCountButton>,
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
            Without<AiPlayerCountButton>,
        ),
    >,
    mut ai_strategy_buttons: Query<
        (&Interaction, &AiStrategyButton),
        (
            Changed<Interaction>,
            With<Button>,
            With<AiStrategyButton>,
            Without<BoardSizeButton>,
            Without<PlayerCountButton>,
            Without<AiPlayerCountButton>,
            Without<AiCooldownButton>,
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
            menu.ai_player_count = menu.ai_player_count.min(menu.player_count);
        }
    }

    for (interaction, button) in &mut ai_player_buttons {
        if *interaction == Interaction::Pressed && button.ai_player_count <= menu.player_count {
            menu.ai_player_count = button.ai_player_count;
        }
    }

    for (interaction, button) in &mut ai_cooldown_buttons {
        if *interaction == Interaction::Pressed {
            menu.ai_cooldown_ms = button.cooldown_ms;
        }
    }

    for (interaction, button) in &mut ai_strategy_buttons {
        if *interaction == Interaction::Pressed {
            menu.ai_strategy = button.strategy;
        }
    }

    for (interaction, button) in &mut network_mode_buttons {
        if *interaction == Interaction::Pressed {
            menu.net_mode = button.mode;
        }
    }
}

pub(super) fn handle_network_connect_button(
    menu: Res<MenuSelection>,
    mut net_config: ResMut<NetConfig>,
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
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn handle_network_address_focus(
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
            Interaction::Hovered if !menu.address_focused => {
                *color = neutral_button_color(Interaction::Hovered).into()
            }
            Interaction::None if !menu.address_focused => {
                *color = neutral_button_color(Interaction::None).into()
            }
            _ => {}
        }
    }

    if clicked_input {
        menu.address_focused = true;
    }
}

pub(super) fn handle_network_address_typing(
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

pub(super) fn sync_menu_layout_visibility(
    menu: Res<MenuSelection>,
    mut sections: Query<(
        Option<&MenuScreenModeSelect>,
        Option<&MenuScreenSetup>,
        Option<&LocalOnly>,
        Option<&NetworkOnly>,
        &mut Node,
    )>,
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
            node.display =
                if menu.screen == MenuScreen::Setup && menu.game_mode == StartGameMode::Local {
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

pub(super) fn sync_menu_button_visuals(
    menu: Res<MenuSelection>,
    net_runtime: Res<NetRuntime>,
    mut option_buttons: Query<
        (
            &Interaction,
            Option<&BoardSizeButton>,
            Option<&PlayerCountButton>,
            Option<&AiPlayerCountButton>,
            Option<&AiCooldownButton>,
            Option<&AiStrategyButton>,
            Option<&ModeChoiceButton>,
            Option<&NetworkModeButton>,
            Option<&NetworkAddressInputButton>,
            &mut BackgroundColor,
        ),
        With<Button>,
    >,
    mut menu_texts: Query<(
        Option<&NetworkAddressText>,
        Option<&ConnectedPlayersText>,
        Option<&StartGameButtonLabel>,
        &mut Text,
    )>,
) {
    for (
        interaction,
        board,
        player,
        ai_player,
        ai_cooldown,
        ai_strategy,
        mode_choice,
        network_mode,
        address_input,
        mut color,
    ) in &mut option_buttons
    {
        *color = if let Some(button) = board {
            selected_button_color(button.radius == menu.board_radius, *interaction).into()
        } else if let Some(button) = player {
            selected_button_color(button.player_count == menu.player_count, *interaction).into()
        } else if let Some(button) = ai_player {
            selected_button_color(button.ai_player_count == menu.ai_player_count, *interaction)
                .into()
        } else if let Some(button) = ai_cooldown {
            selected_button_color(button.cooldown_ms == menu.ai_cooldown_ms, *interaction).into()
        } else if let Some(button) = ai_strategy {
            selected_button_color(button.strategy == menu.ai_strategy, *interaction).into()
        } else if let Some(button) = mode_choice {
            selected_button_color(button.mode == menu.game_mode, *interaction).into()
        } else if let Some(button) = network_mode {
            selected_button_color(button.mode == menu.net_mode, *interaction).into()
        } else if address_input.is_some() {
            selected_button_color(menu.address_focused, *interaction).into()
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
            let label =
                if menu.game_mode == StartGameMode::Network && menu.net_mode == NetMode::Client {
                    "Waiting for Host"
                } else {
                    "Start Game"
                };
            *text = Text::new(label);
        }
    }
}

pub(super) fn handle_start_game_button(
    menu: Res<MenuSelection>,
    mut net_config: ResMut<NetConfig>,
    mut game_config: ResMut<GameConfig>,
    mut app_settings: ResMut<AppSettings>,
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
                if menu.game_mode == StartGameMode::Network {
                    save_last_network_settings(
                        &mut app_settings,
                        menu.net_mode,
                        &net_config.address,
                    );
                }
                game_config.board_radius = menu.board_radius;
                game_config.player_count = if menu.game_mode == StartGameMode::Network {
                    2
                } else {
                    menu.player_count
                };
                game_config.ai_cooldown_seconds = menu.ai_cooldown_ms as f32 / 1_000.0;
                game_config.ai_strategy = if menu.game_mode == StartGameMode::Local {
                    menu.ai_strategy
                } else {
                    AiStrategy::Heuristic
                };
                game_config.player_controls = [PlayerControl::Human; 6];
                if menu.game_mode == StartGameMode::Local {
                    for player_index in 0..menu.ai_player_count {
                        game_config.player_controls[player_index] = PlayerControl::RandomAi;
                    }
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

fn is_valid_address_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | ':' | '-')
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
