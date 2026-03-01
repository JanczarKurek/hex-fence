use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::app_state::{AppPhase, RematchRequested};
use crate::game::state::TurnState;
use crate::settings::{self, AppSettings};

use crate::game::despawn_all;

use super::components::{
    ControlBindingButton, ControlBindingKind, ControlBindingValueText, ExitButton, InGameUiRoot,
    RematchButton, RematchPanel, SettingsPopup, SettingsTab, SettingsTabButton, SettingsTabContent,
    SettingsToggleButton, SettingsUiState, SoundSliderFill, SoundSliderKind, SoundSliderTrack,
    SoundSliderValueText,
};
use super::styles::{
    NORMAL_BUTTON, PANEL_BG, TAB_ACTIVE, TAB_INACTIVE, button_bundle, button_node,
    neutral_button_color, white_text,
};
use super::widgets::{spawn_control_binding_row, spawn_sound_slider_row};

pub(super) fn setup_in_game_ui(
    mut commands: Commands,
    app_settings: Res<AppSettings>,
    settings_ui: Res<SettingsUiState>,
) {
    commands
        .spawn((
            InGameUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
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
                        .spawn(button_bundle(
                            SettingsToggleButton,
                            button_node(120.0, 44.0, 2.0),
                            NORMAL_BUTTON,
                        ))
                        .with_children(|button| {
                            button.spawn(white_text("Settings", 18.0));
                        });

                    top_buttons
                        .spawn(button_bundle(
                            ExitButton,
                            button_node(44.0, 44.0, 2.0),
                            NORMAL_BUTTON,
                        ))
                        .with_children(|button| {
                            button.spawn(white_text("X", 22.0));
                        });
                });

            parent
                .spawn((
                    RematchPanel,
                    Node {
                        width: Val::Auto,
                        height: Val::Auto,
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        display: Display::None,
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(-120.0, -24.0, 0.0)),
                ))
                .with_children(|panel| {
                    panel
                        .spawn(button_bundle(
                            RematchButton,
                            button_node(240.0, 48.0, 2.0),
                            NORMAL_BUTTON,
                        ))
                        .with_children(|button| {
                            button.spawn(white_text("Rematch", 22.0));
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
                                tab.spawn(white_text("Sound", 16.0));
                            });

                            tabs.spawn((
                                Button,
                                SettingsTabButton {
                                    tab: SettingsTab::Controls,
                                },
                                Node {
                                    width: Val::Px(140.0),
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
                                height: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(20.0),
                                ..default()
                            },
                        ))
                        .with_children(|content| {
                            content.spawn(white_text("Audio Mix", 22.0));

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
                                height: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                display: Display::None,
                                ..default()
                            },
                        ))
                        .with_children(|content| {
                            content.spawn(white_text("Controls", 22.0));
                            content.spawn(white_text("Click a binding, then press a key.", 16.0));
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
                });
        });
}

pub(super) fn handle_exit_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ExitButton>),
    >,
    mut next_phase: ResMut<NextState<AppPhase>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            next_phase.set(AppPhase::Menu);
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn cleanup_in_game_ui(
    mut commands: Commands,
    roots: Query<Entity, With<InGameUiRoot>>,
    mut settings_ui: ResMut<SettingsUiState>,
    app_settings: Res<AppSettings>,
) {
    if settings_ui.open {
        let _ = settings::save_settings_to_disk(app_settings.clone());
    }
    settings_ui.pending_control_binding = None;
    despawn_all!(commands, roots);
    settings_ui.open = false;
}

pub(super) fn handle_settings_toggle_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SettingsToggleButton>),
    >,
    mut settings_ui: ResMut<SettingsUiState>,
    app_settings: Res<AppSettings>,
) {
    for (interaction, mut color) in &mut interaction_query {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            let was_open = settings_ui.open;
            settings_ui.open = !settings_ui.open;
            if was_open && !settings_ui.open {
                settings_ui.pending_control_binding = None;
            }
            if was_open && !settings_ui.open {
                let _ = settings::save_settings_to_disk(app_settings.clone());
            }
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn handle_rematch_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RematchButton>),
    >,
    mut rematch_requests: EventWriter<RematchRequested>,
) {
    for (interaction, mut color) in &mut interaction_query {
        let interaction = *interaction;
        if interaction == Interaction::Pressed {
            rematch_requests.write(RematchRequested);
        }
        *color = neutral_button_color(interaction).into();
    }
}

pub(super) fn sync_rematch_visibility(
    turn_state: Res<TurnState>,
    mut panels: Query<&mut Node, With<RematchPanel>>,
) {
    if !turn_state.is_changed() {
        return;
    }

    let display = if turn_state.winner.is_some() {
        Display::Flex
    } else {
        Display::None
    };

    for mut panel in &mut panels {
        panel.display = display;
    }
}

pub(super) fn handle_tab_buttons(
    mut tab_interactions: Query<
        (&Interaction, &SettingsTabButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings_ui: ResMut<SettingsUiState>,
) {
    for (interaction, tab_button) in &mut tab_interactions {
        if *interaction == Interaction::Pressed {
            settings_ui.active_tab = tab_button.tab;
            settings_ui.pending_control_binding = None;
        }
    }
}

pub(super) fn handle_control_binding_buttons(
    mut settings_ui: ResMut<SettingsUiState>,
    interactions: Query<
        (&Interaction, &ControlBindingButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !settings_ui.open || settings_ui.active_tab != SettingsTab::Controls {
        return;
    }

    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            settings_ui.pending_control_binding = Some(button.kind);
        }
    }
}

pub(super) fn handle_control_binding_capture(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings_ui: ResMut<SettingsUiState>,
    mut app_settings: ResMut<AppSettings>,
) {
    if !settings_ui.open || settings_ui.active_tab != SettingsTab::Controls {
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

pub(super) fn sync_control_binding_texts(
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

pub(super) fn sync_control_binding_button_visuals(
    settings_ui: Res<SettingsUiState>,
    mut buttons: Query<(&ControlBindingButton, &Interaction, &mut BackgroundColor), With<Button>>,
) {
    for (button, interaction, mut color) in &mut buttons {
        *color = if settings_ui.pending_control_binding == Some(button.kind) {
            TAB_ACTIVE.into()
        } else {
            neutral_button_color(*interaction).into()
        };
    }
}

pub(super) fn sync_settings_popup_visibility(
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

pub(super) fn handle_sound_slider_input(
    mut app_settings: ResMut<AppSettings>,
    track_query: Query<(&Interaction, &RelativeCursorPosition, &SoundSliderTrack), With<Button>>,
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

pub(super) fn sync_sound_slider_visuals(
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
