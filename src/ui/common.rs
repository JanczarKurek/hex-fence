use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

use crate::settings::{self, AppSettings};

use super::components::{
    ControlBindingValueText, SettingsTab, SettingsTabButton, SettingsTabContent, SettingsUiState,
    SoundSliderFill, SoundSliderTrack, SoundSliderValueText,
};
use super::styles::{TAB_ACTIVE, TAB_INACTIVE};

pub(super) fn save_settings(app_settings: &AppSettings) {
    let _ = settings::save_settings_to_disk(app_settings.clone());
}

pub(super) fn capture_control_binding(
    keys: &ButtonInput<KeyCode>,
    settings_ui: &mut SettingsUiState,
    app_settings: &mut AppSettings,
) {
    let Some(kind) = settings_ui.pending_control_binding else {
        return;
    };

    for key in keys.get_just_pressed() {
        let changed = kind.apply(app_settings, *key);
        settings_ui.pending_control_binding = None;
        if changed {
            save_settings(app_settings);
        }
        break;
    }
}

pub(super) fn sync_control_binding_texts(
    app_settings: &AppSettings,
    settings_ui: &SettingsUiState,
    texts: &mut Query<(&ControlBindingValueText, &mut Text)>,
) {
    for (value_text, mut text) in texts.iter_mut() {
        if settings_ui.pending_control_binding == Some(value_text.kind)
            && settings_ui.active_tab == SettingsTab::Controls
        {
            *text = Text::new("Press key...");
        } else {
            *text = Text::new(value_text.kind.label(app_settings));
        }
    }
}

pub(super) fn sync_settings_tab_ui<F: QueryFilter>(
    settings_ui: &SettingsUiState,
    tab_button_query: &mut Query<(&SettingsTabButton, &mut BackgroundColor), With<Button>>,
    tab_content_query: &mut Query<(&SettingsTabContent, &mut Node), F>,
) {
    for (tab_button, mut tab_color) in tab_button_query.iter_mut() {
        *tab_color = if tab_button.tab == settings_ui.active_tab {
            TAB_ACTIVE.into()
        } else {
            TAB_INACTIVE.into()
        };
    }

    for (tab_content, mut node) in tab_content_query.iter_mut() {
        node.display = if tab_content.tab == settings_ui.active_tab {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn apply_sound_slider_input(
    app_settings: &mut AppSettings,
    track_query: &Query<
        (
            &Interaction,
            &bevy::ui::RelativeCursorPosition,
            &SoundSliderTrack,
        ),
        With<Button>,
    >,
) {
    let mut changed = false;
    for (interaction, cursor_pos, slider) in track_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(normalized) = cursor_pos.normalized else {
            continue;
        };

        slider.kind.set_value(app_settings, normalized.x);
        changed = true;
    }

    if changed {
        save_settings(app_settings);
    }
}

pub(super) fn sync_sound_slider_visuals(
    app_settings: &AppSettings,
    fill_query: &mut Query<(&SoundSliderFill, &mut Node)>,
    value_text_query: &mut Query<(&SoundSliderValueText, &mut Text)>,
) {
    for (fill, mut node) in fill_query.iter_mut() {
        node.width = Val::Percent(fill.kind.value(app_settings) * 100.0);
    }

    for (value_text, mut text) in value_text_query.iter_mut() {
        *text = Text::new(format!(
            "{:>3}%",
            (value_text.kind.value(app_settings) * 100.0).round() as i32
        ));
    }
}
