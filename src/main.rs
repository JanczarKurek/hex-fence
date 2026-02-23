mod app_state;
mod board;
mod camera;
mod game;
mod hex_grid;
mod network;
mod settings;
mod ui;

use app_state::{AppPhase, GameConfig};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::{UpdateMode, WinitSettings};
use core::time::Duration;

fn main() {
    App::new()
        .insert_resource(GameConfig::default())
        .insert_resource(settings::load_settings_from_disk().unwrap_or_default())
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1.0 / 60.0)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(1.0 / 10.0)),
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hex Board".to_string(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppPhase>()
        .add_plugins((
            board::BoardPlugin,
            camera::CameraPlugin,
            network::NetworkPlugin,
            ui::UiPlugin,
            game::GamePlugin,
        ))
        .run();
}
