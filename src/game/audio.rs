use bevy::prelude::*;
use bevy::ecs::world::FromWorld;

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

pub fn start_background_music(mut commands: Commands, audio_assets: Res<GameAudioAssets>) {
    commands.spawn((
        AudioPlayer::new(audio_assets.background_music.clone()),
        PlaybackSettings::LOOP,
    ));
}

pub fn play_sound_effects(
    mut commands: Commands,
    mut sound_events: EventReader<GameSoundEvent>,
    audio_assets: Res<GameAudioAssets>,
) {
    for event in sound_events.read() {
        let source = match event {
            GameSoundEvent::Click => audio_assets.click.clone(),
            GameSoundEvent::SelectPawn => audio_assets.select_pawn.clone(),
            GameSoundEvent::MovePawn => audio_assets.move_pawn.clone(),
            GameSoundEvent::Win => audio_assets.win.clone(),
        };

        commands.spawn((AudioPlayer::new(source), PlaybackSettings::DESPAWN));
    }
}
