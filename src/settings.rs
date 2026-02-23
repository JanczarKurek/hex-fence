use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SETTINGS_FILE_NAME: &str = "settings.toml";
const LEGACY_SETTINGS_PATH: &str = "settings.toml";
const APP_CONFIG_DIR: &str = "giereczka";

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

#[derive(bevy::prelude::Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub audio: AudioSettings,
    #[serde(default)]
    pub network: NetworkSettings,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LastNetMode {
    Host,
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub mode: LastNetMode,
    pub address: String,
    pub local_player_index: usize,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            mode: LastNetMode::Host,
            address: "127.0.0.1:4000".to_string(),
            local_player_index: 0,
        }
    }
}

impl AppSettings {
    fn clamped(self) -> Self {
        Self {
            audio: self.audio.clamped(),
            network: self.network.clamped(),
        }
    }
}

impl NetworkSettings {
    fn clamped(self) -> Self {
        let trimmed = self.address.trim();
        Self {
            mode: self.mode,
            address: if trimmed.is_empty() {
                "127.0.0.1:4000".to_string()
            } else {
                trimmed.chars().take(80).collect()
            },
            local_player_index: self.local_player_index.min(1),
        }
    }
}

pub fn load_settings_from_disk() -> Option<AppSettings> {
    let paths = settings_load_paths();
    for path in paths {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(settings) = toml::from_str::<AppSettings>(&raw) {
            return Some(settings.clamped());
        }

        // Backward compatibility with the old audio-only flat format.
        if let Ok(audio) = toml::from_str::<AudioSettings>(&raw) {
            return Some(
                AppSettings {
                    audio,
                    ..AppSettings::default()
                }
                .clamped(),
            );
        }
    }
    None
}

pub fn save_settings_to_disk(settings: AppSettings) -> io::Result<()> {
    let path = settings_save_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let toml = toml::to_string_pretty(&settings.clamped())
        .map_err(|err| io::Error::other(err.to_string()))?;
    fs::write(path, toml)
}

fn settings_load_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(primary) = xdg_config_settings_path() {
        paths.push(primary);
    }
    paths.push(Path::new(LEGACY_SETTINGS_PATH).to_path_buf());
    paths
}

fn settings_save_path() -> PathBuf {
    xdg_config_settings_path().unwrap_or_else(|| Path::new(LEGACY_SETTINGS_PATH).to_path_buf())
}

fn xdg_config_settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join(APP_CONFIG_DIR).join(SETTINGS_FILE_NAME))
}
