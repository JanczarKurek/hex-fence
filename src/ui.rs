use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::settings::{self, AppSettings};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SettingsUiState::default())
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    handle_exit_button,
                    handle_settings_toggle_button,
                    handle_tab_buttons,
                    handle_sound_slider_input,
                    sync_sound_slider_visuals,
                    sync_settings_popup_visibility,
                ),
            );
    }
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
const PANEL_BG: Color = Color::srgba(0.08, 0.08, 0.1, 0.95);
const TAB_ACTIVE: Color = Color::srgb(0.22, 0.33, 0.44);
const TAB_INACTIVE: Color = Color::srgb(0.13, 0.13, 0.15);
const SLIDER_TRACK: Color = Color::srgb(0.18, 0.18, 0.2);
const SLIDER_FILL: Color = Color::srgb(0.25, 0.68, 0.44);

fn setup_ui(
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
                        margin: UiRect::all(Val::ZERO),
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
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
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
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
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
