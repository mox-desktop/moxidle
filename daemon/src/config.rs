use crate::upower::{BatteryLevel, BatteryState};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};
use tvix_serde::from_str;

#[derive(Deserialize)]
pub struct Config {
    pub general: MoxidleConfig,
    pub listeners: Vec<ListenerConfig>,
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> anyhow::Result<(MoxidleConfig, Vec<ListenerConfig>)> {
        let config_path = if let Some(path) = path {
            path
        } else {
            Self::path()?
        };

        let nix_code = fs::read_to_string(&config_path)?;
        let config: Config =
            from_str(&nix_code).map_err(|e| anyhow::anyhow!("tvix_serde failed: {e:?}"))?;

        Ok((config.general, config.listeners))
    }

    pub fn path() -> anyhow::Result<PathBuf> {
        let home_dir = std::env::var("HOME").map(PathBuf::from)?;
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(".config"))
            .join("mox");

        let config_dir_first = config_dir.join("moxidle.nix");
        if config_dir_first.exists() {
            log::info!("Configuration found at {}", config_dir_first.display());
            return Ok(config_dir_first);
        } else {
            log::warn!("Configuration not found at {}", config_dir_first.display());
        }

        let config_dir_second = config_dir.join("moxidle").join("default.nix");
        if config_dir_second.exists() {
            log::info!("Configuration found at {}", config_dir_second.display());
            Ok(config_dir_second)
        } else {
            log::error!(
                "Configuration not found at {} or {}",
                config_dir_first.display(),
                config_dir_second.display()
            );
            Err(anyhow::anyhow!(
                "Configuration not found at {} or {}",
                config_dir_first.display(),
                config_dir_second.display()
            ))
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct MoxidleConfig {
    pub lock_cmd: Option<Arc<str>>,
    pub unlock_cmd: Option<Arc<str>>,
    pub before_sleep_cmd: Option<Arc<str>>,
    pub after_sleep_cmd: Option<Arc<str>>,
    pub ignore_dbus_inhibit: bool,
    pub ignore_systemd_inhibit: bool,
    #[cfg(feature = "audio")]
    pub ignore_audio_inhibit: bool,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    OnBattery,
    OnAc,
    BatteryBelow(f64),
    BatteryAbove(f64),
    BatteryEqual(f64),
    BatteryLevel(BatteryLevel),
    BatteryState(BatteryState),
    UsbPlugged(Arc<str>),
    UsbUnplugged(Arc<str>),
}

#[derive(Deserialize)]
pub struct ListenerConfig {
    #[serde(default)]
    pub conditions: Box<[Condition]>,
    pub timeout: u32,
    pub on_timeout: Option<Arc<str>>,
    pub on_resume: Option<Arc<str>>,
}

impl ListenerConfig {
    pub fn timeout_millis(&self) -> u32 {
        self.timeout * 1000
    }
}
