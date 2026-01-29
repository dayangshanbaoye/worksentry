use crate::commands::{Config, HotkeyConfig, default_browser_search};
use crate::services::browser_extractor;
use std::fs;
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| Mutex::new(load_config()));

fn load_config() -> Config {
    let config_path = get_config_path();
    if let Ok(path) = config_path {
        if let Ok(json) = fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<Config>(&json) {
                return config;
            }
        }
    }
    Config::default()
}

pub fn get_config_path() -> Result<std::path::PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Unable to get config directory")?
        .join("worksentry");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    Ok(config_dir.join("config.json"))
}

pub fn get_config() -> Result<Config, String> {
    let config = CONFIG.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

pub fn set_hotkey(modifiers: Vec<String>, key: String) -> Result<(), String> {
    let mut config = CONFIG.lock().map_err(|e| e.to_string())?;
    config.hotkey = HotkeyConfig { modifiers, key };
    save_config(&config)?;
    Ok(())
}

pub fn set_browser_enabled(enabled: bool) -> Result<(), String> {
    let mut config = CONFIG.lock().map_err(|e| e.to_string())?;
    config.enable_browser_search = enabled;
    save_config(&config)?;
    Ok(())
}

pub fn get_browser_status() -> Result<crate::services::browser_extractor::BrowserStatus, String> {
    let installed = browser_extractor::get_installed_browsers();
    Ok(crate::services::browser_extractor::BrowserStatus {
        installed_browsers: installed,
    })
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path()?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path, json).map_err(|e| e.to_string())?;
    Ok(())
}
