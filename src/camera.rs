use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, zoom_camera);
    }
}

#[derive(Component)]
pub struct MainCamera;

const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 3.0;
const ZOOM_STEP: f32 = 0.1;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn zoom_camera(
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Projection, With<MainCamera>>,
) {
    let mut scroll_delta: f32 = 0.0;
    for event in scroll_events.read() {
        scroll_delta += event.y;
    }

    if scroll_delta.abs() <= f32::EPSILON {
        return;
    }

    for mut projection in &mut query {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            let zoom_change = 1.0 - (scroll_delta * ZOOM_STEP);
            if zoom_change <= 0.0 {
                continue;
            }
            ortho.scale = (ortho.scale * zoom_change).clamp(MIN_ZOOM, MAX_ZOOM);
        }
    }
}
