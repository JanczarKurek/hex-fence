use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

const SETTINGS_PATH: &str = "settings.toml";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master: f32,
    pub music: f32,
    pub effects: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: 1.0,
            music: 0.7,
            effects: 0.8,
        }
    }
}

impl AudioSettings {
    fn clamped(self) -> Self {
        Self {
            master: self.master.clamp(0.0, 1.0),
            music: self.music.clamp(0.0, 1.0),
            effects: self.effects.clamp(0.0, 1.0),
        }
    }

    pub fn effective_music_volume(self) -> f32 {
        (self.master * self.music).clamp(0.0, 1.0)
    }

    pub fn effective_effects_volume(self) -> f32 {
        (self.master * self.effects).clamp(0.0, 1.0)
    }
}

#[derive(bevy::prelude::Resource, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub audio: AudioSettings,
}

impl AppSettings {
    fn clamped(self) -> Self {
        Self {
            audio: self.audio.clamped(),
        }
    }
}

pub fn load_settings_from_disk() -> Option<AppSettings> {
    let path = Path::new(SETTINGS_PATH);
    let raw = fs::read_to_string(path).ok()?;
    if let Ok(settings) = toml::from_str::<AppSettings>(&raw) {
        return Some(settings.clamped());
    }

    // Backward compatibility with the old audio-only flat format.
    toml::from_str::<AudioSettings>(&raw)
        .ok()
        .map(|audio| AppSettings { audio }.clamped())
}

pub fn save_settings_to_disk(settings: AppSettings) -> io::Result<()> {
    let path = Path::new(SETTINGS_PATH);
    let toml = toml::to_string_pretty(&settings.clamped())
        .map_err(|err| io::Error::other(err.to_string()))?;
    fs::write(path, toml)
}
