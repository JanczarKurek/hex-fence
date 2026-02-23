mod components;
mod in_game_menu;
mod start_menu;
mod styles;
mod widgets;

use bevy::prelude::*;

use crate::app_state::AppPhase;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(components::SettingsUiState::default())
            .insert_resource(components::MenuSelection::default())
            .add_systems(OnEnter(AppPhase::Menu), start_menu::setup_start_menu)
            .add_systems(OnExit(AppPhase::Menu), start_menu::cleanup_start_menu)
            .add_systems(
                Update,
                (
                    start_menu::handle_mode_choice_buttons,
                    start_menu::handle_back_to_mode_button,
                    start_menu::handle_menu_option_buttons,
                    start_menu::handle_network_connect_button,
                    start_menu::handle_network_address_focus,
                    start_menu::handle_network_address_typing,
                    start_menu::handle_start_game_button,
                    start_menu::sync_menu_layout_visibility,
                    start_menu::sync_menu_button_visuals,
                )
                    .run_if(in_state(AppPhase::Menu)),
            )
            .add_systems(OnEnter(AppPhase::InGame), in_game_menu::setup_in_game_ui)
            .add_systems(OnExit(AppPhase::InGame), in_game_menu::cleanup_in_game_ui)
            .add_systems(
                Update,
                (
                    in_game_menu::handle_exit_button,
                    in_game_menu::handle_rematch_button,
                    in_game_menu::handle_settings_toggle_button,
                    in_game_menu::handle_tab_buttons,
                    in_game_menu::handle_sound_slider_input,
                    in_game_menu::sync_sound_slider_visuals,
                    in_game_menu::sync_rematch_visibility,
                    in_game_menu::sync_settings_popup_visibility,
                )
                    .run_if(in_state(AppPhase::InGame)),
            );
    }
}
