mod board;
mod camera;
mod game;
mod hex_grid;
mod settings;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(settings::load_settings_from_disk().unwrap_or_default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hex Board".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            board::BoardPlugin,
            camera::CameraPlugin,
            ui::UiPlugin,
            game::GamePlugin,
        ))
        .run();
}
