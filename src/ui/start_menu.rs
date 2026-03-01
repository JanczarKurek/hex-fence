use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_simple_text_input::{TextInput, TextInputInactive, TextInputValue};

use crate::app_state::{AiStrategy, AppPhase, GameConfig, PlayerControl};
use crate::network::{NetConfig, NetMode, NetRuntime};
use crate::settings::{self, AppSettings, LastNetMode};

use crate::game::despawn_all;

use super::components::{
    AiCooldownButton, AiPlayerCountButton, AiStrategyButton, AuthorsPopup, BackToModeButton,
    BoardSizeButton, ConnectedPlayersText, ControlBindingButton, ControlBindingKind,
    ControlBindingValueText, LocalOnly, MainMenuAction, MainMenuActionButton, MenuScreen,
    MenuScreenModeSelect, MenuScreenSetup, MenuSelection, MenuSettingsCloseButton,
    MenuSettingsPopup, NetworkAddressInputButton, NetworkAddressInputField, NetworkConnectButton,
    NetworkModeButton, NetworkOnly, PlayerCountButton, RulesPopup, SettingsTab, SettingsTabButton,
    SettingsTabContent, SettingsUiState, SoundSliderFill, SoundSliderKind, SoundSliderTrack,
    SoundSliderValueText, StartGameButton, StartGameButtonLabel, StartGameMode, StartMenuRoot,
};
use super::styles::{
    HOVERED_BUTTON, MENU_PANEL_BG, MENU_SELECTED, MENU_START, NORMAL_BUTTON, PRESSED_BUTTON,
    TAB_ACTIVE, TAB_INACTIVE, button_bundle, button_node, menu_text, neutral_button_color,
    selected_button_color, white_text,
};
use super::widgets::{
    spawn_ai_cooldown_row, spawn_ai_player_row, spawn_ai_strategy_row, spawn_choice_row,
    spawn_control_binding_row, spawn_network_mode_row, spawn_player_row, spawn_sound_slider_row,
};

const AI_COOLDOWN_CHOICES_MS: [u32; 5] = [250, 500, 1_000, 1_500, 2_000];

pub(super) fn setup_start_menu(
    mut commands: Commands,
    game_config: Res<GameConfig>,
    app_settings: Res<AppSettings>,
    net_config: Res<NetConfig>,
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
    menu.show_authors_popup = false;
    menu.show_rules_popup = false;
    menu.show_settings_popup = false;

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

            });

            root
                .spawn((
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

            root
                .spawn((
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

            root
                .spawn((
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
                                    content.spawn(menu_text("Click a binding, then press a key.", 16.0));
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
            menu.screen = MenuScreen::ModeSelect;
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
    interactions: Query<(&Interaction, &ControlBindingButton), (Changed<Interaction>, With<Button>)>,
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
    track_query: Query<(&Interaction, &bevy::ui::RelativeCursorPosition, &SoundSliderTrack), With<Button>>,
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
    mut sections: Query<(
        Option<&MenuScreenModeSelect>,
        Option<&MenuScreenSetup>,
        Option<&AuthorsPopup>,
        Option<&RulesPopup>,
        Option<&MenuSettingsPopup>,
        Option<&LocalOnly>,
        Option<&NetworkOnly>,
        &mut Node,
    )>,
) {
    if !menu.is_changed() {
        return;
    }

    for (mode_screen, setup_screen, authors_popup, rules_popup, settings_popup, local_only, network_only, mut node) in &mut sections {
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
    settings_ui: Res<SettingsUiState>,
    net_runtime: Res<NetRuntime>,
    mut option_buttons: Query<
        (
            &Interaction,
            Option<&BoardSizeButton>,
            Option<&PlayerCountButton>,
            Option<&AiPlayerCountButton>,
            Option<&AiCooldownButton>,
            Option<&AiStrategyButton>,
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
        ai_player,
        ai_cooldown,
        ai_strategy,
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
        } else if let Some(button) = ai_player {
            selected_button_color(button.ai_player_count == menu.ai_player_count, *interaction)
                .into()
        } else if let Some(button) = ai_cooldown {
            selected_button_color(button.cooldown_ms == menu.ai_cooldown_ms, *interaction).into()
        } else if let Some(button) = ai_strategy {
            selected_button_color(button.strategy == menu.ai_strategy, *interaction).into()
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

fn control_binding_label(app_settings: &AppSettings, binding_kind: ControlBindingKind) -> &'static str {
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
