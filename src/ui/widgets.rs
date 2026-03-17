use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::network::NetMode;

use super::components::{
    AiCooldownButton, BoardSizeButton, ControlBindingButton, ControlBindingKind,
    ControlBindingValueText, MenuSettingsCloseButton, NetworkModeButton, PlayerCountButton,
    SettingsTab, SettingsTabButton, SoundSliderFill, SoundSliderKind, SoundSliderTrack,
    SoundSliderValueText,
};
use super::styles::{
    MENU_SELECTED, NORMAL_BUTTON, SLIDER_FILL, SLIDER_TRACK, TAB_ACTIVE, TAB_INACTIVE, VALUE_TEXT,
    button_bundle, button_node, column_node, row_node, selected_button_color, tab_button_node,
    text_bundle, white_text,
};

pub(super) fn spawn_text_button<C: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: C,
    label: impl Into<String>,
    node: Node,
    background: Color,
    font_size: f32,
) {
    parent
        .spawn(button_bundle(marker, node, background))
        .with_children(|button| {
            button.spawn(white_text(label, font_size));
        });
}

pub(super) fn spawn_close_button(parent: &mut ChildSpawnerCommands) {
    spawn_text_button(
        parent,
        MenuSettingsCloseButton,
        "Close",
        button_node(120.0, 36.0, 1.0),
        NORMAL_BUTTON,
        16.0,
    );
}

pub(super) fn spawn_settings_tabs(parent: &mut ChildSpawnerCommands, width_px: f32) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|tabs| {
            for (tab, label, active) in [
                (SettingsTab::Sound, "Sound", true),
                (SettingsTab::Controls, "Controls", false),
            ] {
                tabs.spawn((
                    Button,
                    SettingsTabButton { tab },
                    tab_button_node(width_px),
                    BackgroundColor(if active { TAB_ACTIVE } else { TAB_INACTIVE }),
                ))
                .with_children(|button| {
                    button.spawn(white_text(label, 16.0));
                });
            }
        });
}

pub(super) fn settings_content_node(row_gap_px: f32) -> Node {
    column_node(row_gap_px)
}

pub(super) fn spawn_choice_row(parent: &mut ChildSpawnerCommands, choices: &[i32], selected: i32) {
    parent.spawn(row_node(8.0)).with_children(|row| {
        for choice in choices {
            row.spawn(button_bundle(
                BoardSizeButton { radius: *choice },
                button_node(72.0, 38.0, 1.0),
                if *choice == selected {
                    MENU_SELECTED
                } else {
                    NORMAL_BUTTON
                },
            ))
            .with_children(|button| {
                button.spawn(white_text(choice.to_string(), 18.0));
            });
        }
    });
}

pub(super) fn spawn_player_row(
    parent: &mut ChildSpawnerCommands,
    choices: &[usize],
    selected: usize,
) {
    parent.spawn(row_node(8.0)).with_children(|row| {
        for choice in choices {
            row.spawn(button_bundle(
                PlayerCountButton {
                    player_count: *choice,
                },
                button_node(72.0, 38.0, 1.0),
                if *choice == selected {
                    MENU_SELECTED
                } else {
                    NORMAL_BUTTON
                },
            ))
            .with_children(|button| {
                button.spawn(white_text(choice.to_string(), 18.0));
            });
        }
    });
}

pub(super) fn spawn_ai_cooldown_row(parent: &mut ChildSpawnerCommands, selected_ms: u32) {
    const COOLDOWN_CHOICES_MS: [u32; 5] = [250, 500, 1_000, 1_500, 2_000];

    parent.spawn(row_node(8.0)).with_children(|row| {
        for cooldown_ms in COOLDOWN_CHOICES_MS {
            row.spawn(button_bundle(
                AiCooldownButton { cooldown_ms },
                button_node(72.0, 38.0, 1.0),
                if cooldown_ms == selected_ms {
                    MENU_SELECTED
                } else {
                    NORMAL_BUTTON
                },
            ))
            .with_children(|button| {
                let label = if cooldown_ms < 1_000 {
                    format!("{}ms", cooldown_ms)
                } else if cooldown_ms % 1_000 == 0 {
                    format!("{}s", cooldown_ms / 1_000)
                } else {
                    format!("{:.1}s", cooldown_ms as f32 / 1_000.0)
                };
                button.spawn(white_text(label, 14.0));
            });
        }
    });
}

pub(super) fn spawn_network_mode_row(parent: &mut ChildSpawnerCommands, selected: NetMode) {
    parent.spawn(row_node(8.0)).with_children(|row| {
        for (label, mode) in [("Host", NetMode::Host), ("Client", NetMode::Client)] {
            row.spawn(button_bundle(
                NetworkModeButton { mode },
                button_node(96.0, 38.0, 1.0),
                selected_button_color(mode == selected, Interaction::None),
            ))
            .with_children(|button| {
                button.spawn(white_text(label, 16.0));
            });
        }
    });
}

pub(super) fn spawn_sound_slider_row(
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
                white_text(label, 16.0),
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
                text_bundle(
                    format!("{:>3}%", (clamped * 100.0).round() as i32),
                    16.0,
                    VALUE_TEXT,
                ),
                Node {
                    width: Val::Px(56.0),
                    ..default()
                },
            ));
        });
}

pub(super) fn spawn_control_binding_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    kind: ControlBindingKind,
    value: &str,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                white_text(label, 16.0),
                Node {
                    width: Val::Px(230.0),
                    ..default()
                },
            ));

            row.spawn(button_bundle(
                ControlBindingButton { kind },
                Node {
                    width: Val::Px(140.0),
                    height: Val::Px(34.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                NORMAL_BUTTON,
            ))
            .with_children(|button| {
                button.spawn((ControlBindingValueText { kind }, white_text(value, 15.0)));
            });
        });
}
