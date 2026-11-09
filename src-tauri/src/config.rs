//! Config persistence: %APPDATA%\Oryvn\config.json
//! - version field + migrations
//! - atomic writes (write temp, rename)
//! - never crash on corrupt config: back it up and load defaults

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::brand::APP_NAME;
use crate::dock_model::AppConfig;

const CONFIG_VERSION: u32 = 1;

fn app_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data = std::env::var("APPDATA").map_err(|_| "APPDATA not set")?;
    let dir = PathBuf::from(app_data).join(APP_NAME);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(app_dir()?.join("config.json"))
}

fn backup_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(app_dir()?.join("config.json.bak"))
}

fn log_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = app_dir()?.join("logs");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Load config. Never crashes: on corrupt/missing file, back it up and return defaults.
pub fn load_config() -> AppConfig {
    match load_config_inner() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::warn!("Failed to load config ({}), loading defaults", e);
            // Back up the corrupt file if it exists.
            if let Ok(p) = config_path() {
                if p.exists() {
                    if let Ok(bp) = backup_path() {
                        let _ = fs::copy(&p, &bp);
                        tracing::warn!("Backed up corrupt config to {:?}", bp);
                    }
                }
            }
            AppConfig::default()
        }
    }
}

fn load_config_inner() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(&path)?;
    let mut cfg: AppConfig = serde_json::from_str(&raw)?;
    cfg = migrate(cfg)?;
    Ok(cfg)
}

/// Save config atomically: write temp file in same dir, then rename.
pub fn save_config(cfg: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path()?;
    let dir = path.parent().unwrap_or(std::path::Path::new(""));
    let tmp = dir.join(format!("{}.tmp", APP_NAME));

    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Reset config to defaults (and back up current).
pub fn reset_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    if let Ok(p) = config_path() {
        if p.exists() {
            if let Ok(bp) = backup_path() {
                let _ = fs::rename(&p, &bp);
            }
        }
    }
    let default = AppConfig::default();
    save_config(&default)?;
    Ok(default)
}

fn migrate(mut cfg: AppConfig) -> Result<AppConfig, Box<dyn std::error::Error>> {
    match cfg.version {
        0 => {
            // v0 -> v1: initialize empty docks/items if missing.
            cfg.version = CONFIG_VERSION;
            Ok(cfg)
        }
        v if v == CONFIG_VERSION => Ok(cfg),
        v => {
            tracing::warn!("Unknown config version {}, loading defaults", v);
            Ok(AppConfig::default())
        }
    }
}

pub fn log_dir_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    log_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_serializes() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.version, back.version);
    }

    #[test]
    fn test_migrate_v0_to_v1() {
        let mut cfg = AppConfig::default();
        cfg.version = 0;
        let migrated = migrate(cfg).unwrap();
        assert_eq!(migrated.version, CONFIG_VERSION);
    }

    #[test]
    fn test_dock_items_default_empty() {
        let cfg = AppConfig::default();
        assert!(cfg.items_for("nonexistent").is_empty());
        assert!(cfg.docks.is_empty());
    }
}
