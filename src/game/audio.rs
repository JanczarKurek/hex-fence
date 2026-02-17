use bevy::audio::{AudioSinkPlayback, Volume};
use bevy::ecs::world::FromWorld;
use bevy::prelude::*;

use crate::settings::AppSettings;

#[derive(Event, Clone, Copy)]
pub enum GameSoundEvent {
    Click,
    SelectPawn,
    MovePawn,
    Win,
}

#[derive(Resource)]
pub struct GameAudioAssets {
    click: Handle<AudioSource>,
    select_pawn: Handle<AudioSource>,
    move_pawn: Handle<AudioSource>,
    win: Handle<AudioSource>,
    background_music: Handle<AudioSource>,
}

impl FromWorld for GameAudioAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        Self {
            click: asset_server.load("music/click.ogg"),
            select_pawn: asset_server.load("music/select_pawn.ogg"),
            move_pawn: asset_server.load("music/move.ogg"),
            win: asset_server.load("music/win.ogg"),
            background_music: asset_server.load("music/background.ogg"),
        }
    }
}

#[derive(Component)]
pub struct BackgroundMusic;

pub fn start_background_music(
    mut commands: Commands,
    audio_assets: Res<GameAudioAssets>,
    app_settings: Res<AppSettings>,
) {
    commands.spawn((
        BackgroundMusic,
        AudioPlayer::new(audio_assets.background_music.clone()),
        PlaybackSettings::LOOP
            .with_volume(Volume::Linear(app_settings.audio.effective_music_volume())),
    ));
}

pub fn play_sound_effects(
    mut commands: Commands,
    mut sound_events: EventReader<GameSoundEvent>,
    audio_assets: Res<GameAudioAssets>,
    app_settings: Res<AppSettings>,
) {
    for event in sound_events.read() {
        let source = match event {
            GameSoundEvent::Click => audio_assets.click.clone(),
            GameSoundEvent::SelectPawn => audio_assets.select_pawn.clone(),
            GameSoundEvent::MovePawn => audio_assets.move_pawn.clone(),
            GameSoundEvent::Win => audio_assets.win.clone(),
        };

        commands.spawn((
            AudioPlayer::new(source),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(
                app_settings.audio.effective_effects_volume(),
            )),
        ));
    }
}

pub fn update_background_music_volume(
    app_settings: Res<AppSettings>,
    mut music_sinks: Query<&mut AudioSink, With<BackgroundMusic>>,
) {
    if !app_settings.is_changed() {
        return;
    }

    let volume = Volume::Linear(app_settings.audio.effective_music_volume());
    for mut sink in &mut music_sinks {
        sink.set_volume(volume);
    }
}
