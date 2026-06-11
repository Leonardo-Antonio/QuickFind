use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub max_results: usize,
    pub window_width: i32,
    pub window_height: i32,
    pub icon_size: i32,
    pub show_icons: bool,
    pub terminal_emulator: String,
    pub launch_on_single_result: bool,
    pub cache_apps: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_results: 10,
            window_width: 640,
            window_height: 420,
            icon_size: 48,
            show_icons: true,
            terminal_emulator: String::from("kitty"),
            launch_on_single_result: true,
            cache_apps: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        }
        Config::default()
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("quickfind")
            .join("config.toml")
    }

    pub fn ensure_default() {
        let path = Self::config_path();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let default = Config::default();
            if let Ok(content) = toml::to_string_pretty(&default) {
                let _ = std::fs::write(&path, content);
            }
        }
    }
}
