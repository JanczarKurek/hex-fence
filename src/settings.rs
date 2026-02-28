use bevy::prelude::KeyCode;
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
    #[serde(default)]
    pub controls: ControlsSettings,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlsSettings {
    pub toggle_fence_mode: String,
    pub cycle_fence_shape: String,
    pub rotate_fence_orientation: String,
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

impl Default for ControlsSettings {
    fn default() -> Self {
        Self {
            toggle_fence_mode: "F".to_string(),
            cycle_fence_shape: "Q".to_string(),
            rotate_fence_orientation: "E".to_string(),
        }
    }
}

impl AppSettings {
    fn clamped(self) -> Self {
        Self {
            audio: self.audio.clamped(),
            network: self.network.clamped(),
            controls: self.controls.clamped(),
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

impl ControlsSettings {
    fn clamped(self) -> Self {
        let mut clamped = Self::default();

        if let Some(key) = key_code_from_label(&self.toggle_fence_mode) {
            let _ = clamped.set_toggle_fence_mode_key(key);
        }
        if let Some(key) = key_code_from_label(&self.cycle_fence_shape) {
            let _ = clamped.set_cycle_fence_shape_key(key);
        }
        if let Some(key) = key_code_from_label(&self.rotate_fence_orientation) {
            let _ = clamped.set_rotate_fence_orientation_key(key);
        }

        clamped
    }

    pub fn toggle_fence_mode_key(&self) -> KeyCode {
        key_code_from_label(&self.toggle_fence_mode).unwrap_or(KeyCode::KeyF)
    }

    pub fn cycle_fence_shape_key(&self) -> KeyCode {
        key_code_from_label(&self.cycle_fence_shape).unwrap_or(KeyCode::KeyQ)
    }

    pub fn rotate_fence_orientation_key(&self) -> KeyCode {
        key_code_from_label(&self.rotate_fence_orientation).unwrap_or(KeyCode::KeyE)
    }

    pub fn toggle_fence_mode_label(&self) -> &'static str {
        key_code_label(self.toggle_fence_mode_key()).unwrap_or("F")
    }

    pub fn cycle_fence_shape_label(&self) -> &'static str {
        key_code_label(self.cycle_fence_shape_key()).unwrap_or("Q")
    }

    pub fn rotate_fence_orientation_label(&self) -> &'static str {
        key_code_label(self.rotate_fence_orientation_key()).unwrap_or("E")
    }

    pub fn set_toggle_fence_mode_key(&mut self, key: KeyCode) -> bool {
        let Some(label) = key_code_label(key) else {
            return false;
        };
        self.toggle_fence_mode = label.to_string();
        true
    }

    pub fn set_cycle_fence_shape_key(&mut self, key: KeyCode) -> bool {
        let Some(label) = key_code_label(key) else {
            return false;
        };
        self.cycle_fence_shape = label.to_string();
        true
    }

    pub fn set_rotate_fence_orientation_key(&mut self, key: KeyCode) -> bool {
        let Some(label) = key_code_label(key) else {
            return false;
        };
        self.rotate_fence_orientation = label.to_string();
        true
    }
}

fn key_code_from_label(raw: &str) -> Option<KeyCode> {
    match raw.trim().to_uppercase().as_str() {
        "A" => Some(KeyCode::KeyA),
        "B" => Some(KeyCode::KeyB),
        "C" => Some(KeyCode::KeyC),
        "D" => Some(KeyCode::KeyD),
        "E" => Some(KeyCode::KeyE),
        "F" => Some(KeyCode::KeyF),
        "G" => Some(KeyCode::KeyG),
        "H" => Some(KeyCode::KeyH),
        "I" => Some(KeyCode::KeyI),
        "J" => Some(KeyCode::KeyJ),
        "K" => Some(KeyCode::KeyK),
        "L" => Some(KeyCode::KeyL),
        "M" => Some(KeyCode::KeyM),
        "N" => Some(KeyCode::KeyN),
        "O" => Some(KeyCode::KeyO),
        "P" => Some(KeyCode::KeyP),
        "Q" => Some(KeyCode::KeyQ),
        "R" => Some(KeyCode::KeyR),
        "S" => Some(KeyCode::KeyS),
        "T" => Some(KeyCode::KeyT),
        "U" => Some(KeyCode::KeyU),
        "V" => Some(KeyCode::KeyV),
        "W" => Some(KeyCode::KeyW),
        "X" => Some(KeyCode::KeyX),
        "Y" => Some(KeyCode::KeyY),
        "Z" => Some(KeyCode::KeyZ),
        "0" => Some(KeyCode::Digit0),
        "1" => Some(KeyCode::Digit1),
        "2" => Some(KeyCode::Digit2),
        "3" => Some(KeyCode::Digit3),
        "4" => Some(KeyCode::Digit4),
        "5" => Some(KeyCode::Digit5),
        "6" => Some(KeyCode::Digit6),
        "7" => Some(KeyCode::Digit7),
        "8" => Some(KeyCode::Digit8),
        "9" => Some(KeyCode::Digit9),
        "SPACE" => Some(KeyCode::Space),
        "TAB" => Some(KeyCode::Tab),
        "ENTER" => Some(KeyCode::Enter),
        "ESC" | "ESCAPE" => Some(KeyCode::Escape),
        "UP" => Some(KeyCode::ArrowUp),
        "DOWN" => Some(KeyCode::ArrowDown),
        "LEFT" => Some(KeyCode::ArrowLeft),
        "RIGHT" => Some(KeyCode::ArrowRight),
        _ => None,
    }
}

fn key_code_label(key: KeyCode) -> Option<&'static str> {
    match key {
        KeyCode::KeyA => Some("A"),
        KeyCode::KeyB => Some("B"),
        KeyCode::KeyC => Some("C"),
        KeyCode::KeyD => Some("D"),
        KeyCode::KeyE => Some("E"),
        KeyCode::KeyF => Some("F"),
        KeyCode::KeyG => Some("G"),
        KeyCode::KeyH => Some("H"),
        KeyCode::KeyI => Some("I"),
        KeyCode::KeyJ => Some("J"),
        KeyCode::KeyK => Some("K"),
        KeyCode::KeyL => Some("L"),
        KeyCode::KeyM => Some("M"),
        KeyCode::KeyN => Some("N"),
        KeyCode::KeyO => Some("O"),
        KeyCode::KeyP => Some("P"),
        KeyCode::KeyQ => Some("Q"),
        KeyCode::KeyR => Some("R"),
        KeyCode::KeyS => Some("S"),
        KeyCode::KeyT => Some("T"),
        KeyCode::KeyU => Some("U"),
        KeyCode::KeyV => Some("V"),
        KeyCode::KeyW => Some("W"),
        KeyCode::KeyX => Some("X"),
        KeyCode::KeyY => Some("Y"),
        KeyCode::KeyZ => Some("Z"),
        KeyCode::Digit0 => Some("0"),
        KeyCode::Digit1 => Some("1"),
        KeyCode::Digit2 => Some("2"),
        KeyCode::Digit3 => Some("3"),
        KeyCode::Digit4 => Some("4"),
        KeyCode::Digit5 => Some("5"),
        KeyCode::Digit6 => Some("6"),
        KeyCode::Digit7 => Some("7"),
        KeyCode::Digit8 => Some("8"),
        KeyCode::Digit9 => Some("9"),
        KeyCode::Space => Some("SPACE"),
        KeyCode::Tab => Some("TAB"),
        KeyCode::Enter => Some("ENTER"),
        KeyCode::Escape => Some("ESC"),
        KeyCode::ArrowUp => Some("UP"),
        KeyCode::ArrowDown => Some("DOWN"),
        KeyCode::ArrowLeft => Some("LEFT"),
        KeyCode::ArrowRight => Some("RIGHT"),
        _ => None,
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
