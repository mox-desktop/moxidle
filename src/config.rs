use mlua::{Lua, LuaSerdeExt};
use serde::{Deserialize, Deserializer};
use std::{fs, path::PathBuf, sync::Arc};

use crate::upower::{BatteryLevel, BatteryState};

#[derive(Deserialize)]
pub struct Config {
    pub general: MoxidleConfig,
    pub timeouts: Vec<TimeoutConfig>,
}

impl Config {
    pub fn load(
        path: Option<PathBuf>,
    ) -> Result<(MoxidleConfig, Vec<TimeoutConfig>), Box<dyn std::error::Error>> {
        let config_path = if let Some(path) = path {
            path
        } else {
            Self::path()?
        };
        let lua_code = fs::read_to_string(&config_path)?;
        let lua = Lua::new();
        let lua_result = lua.load(&lua_code).eval()?;

        let config: Config = lua.from_value(lua_result)?;

        Ok((config.general, config.timeouts))
    }

    pub fn path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".config")))?;

        Ok(config_dir.join("moxidle/config.lua"))
    }
}

#[derive(Deserialize)]
pub struct MoxidleConfig {
    pub lock_cmd: Option<Arc<str>>,
    pub unlock_cmd: Option<Arc<str>>,
    pub before_sleep_cmd: Option<Arc<str>>,
    pub after_sleep_cmd: Option<Arc<str>>,
    #[serde(default)]
    pub ignore_dbus_inhibit: bool,
    #[serde(default)]
    pub ignore_systemd_inhibit: bool,
    #[serde(default)]
    #[cfg(feature = "audio")]
    pub ignore_audio_inhibit: bool,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    OnBattery,
    OnAc,
    BatteryBelow(f64),
    BatteryAbove(f64),
    BatteryEqual(f64),
    #[serde(deserialize_with = "deserialize_battery_level")]
    BatteryLevel(BatteryLevel),
    #[serde(deserialize_with = "deserialize_battery_state")]
    BatteryState(BatteryState),
}

#[derive(Debug)]
pub struct InvalidBatteryStateError;

impl TryFrom<&str> for BatteryState {
    type Error = InvalidBatteryStateError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "unknown" => Ok(BatteryState::Unknown),
            "charging" => Ok(BatteryState::Charging),
            "discharging" => Ok(BatteryState::Discharging),
            "empty" => Ok(BatteryState::Empty),
            "fully_charged" => Ok(BatteryState::FullyCharged),
            "pending_charge" => Ok(BatteryState::PendingCharge),
            "pending_discharge" => Ok(BatteryState::PendingDischarge),
            _ => Err(InvalidBatteryStateError),
        }
    }
}

fn deserialize_battery_state<'de, D>(deserializer: D) -> Result<BatteryState, D::Error>
where
    D: Deserializer<'de>,
{
    struct BatteryStateVisitor;

    impl serde::de::Visitor<'_> for BatteryStateVisitor {
        type Value = BatteryState;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer (u32) or a snake_case string")
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value {
                0 => Ok(BatteryState::Unknown),
                1 => Ok(BatteryState::Charging),
                2 => Ok(BatteryState::Discharging),
                3 => Ok(BatteryState::Empty),
                4 => Ok(BatteryState::FullyCharged),
                5 => Ok(BatteryState::PendingCharge),
                6 => Ok(BatteryState::PendingDischarge),
                _ => Err(E::custom(format!("Invalid BatteryState u32: {}", value))),
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            BatteryState::try_from(value)
                .map_err(|_| E::custom(format!("Invalid BatteryState string: {}", value)))
        }
    }

    deserializer.deserialize_any(BatteryStateVisitor)
}

#[derive(Debug)]
pub struct InvalidBatteryLevelError;

impl TryFrom<&str> for BatteryLevel {
    type Error = InvalidBatteryLevelError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "unknown" => Ok(BatteryLevel::Unknown),
            "none" => Ok(BatteryLevel::None),
            "low" => Ok(BatteryLevel::Low),
            "critical" => Ok(BatteryLevel::Critical),
            "normal" => Ok(BatteryLevel::Normal),
            "high" => Ok(BatteryLevel::High),
            "full" => Ok(BatteryLevel::Full),
            _ => Err(InvalidBatteryLevelError),
        }
    }
}

fn deserialize_battery_level<'de, D>(deserializer: D) -> Result<BatteryLevel, D::Error>
where
    D: Deserializer<'de>,
{
    struct BatteryLevelVisitor;

    impl serde::de::Visitor<'_> for BatteryLevelVisitor {
        type Value = BatteryLevel;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer (u32) or a snake_case string")
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value {
                0 => Ok(BatteryLevel::Unknown),
                1 => Ok(BatteryLevel::None),
                2 => Ok(BatteryLevel::Low),
                3 => Ok(BatteryLevel::Critical),
                4 => Ok(BatteryLevel::Normal),
                5 => Ok(BatteryLevel::High),
                6 => Ok(BatteryLevel::Full),
                _ => Err(E::custom(format!("Invalid BatteryLevel u32: {}", value))),
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            BatteryLevel::try_from(value)
                .map_err(|_| E::custom(format!("Invalid BatteryState string: {}", value)))
        }
    }

    deserializer.deserialize_any(BatteryLevelVisitor)
}

#[derive(Deserialize)]
pub struct TimeoutConfig {
    #[serde(default)]
    pub conditions: Arc<[Condition]>,
    pub timeout: u32,
    pub on_timeout: Option<Arc<str>>,
    pub on_resume: Option<Arc<str>>,
}

impl TimeoutConfig {
    pub fn timeout_millis(&self) -> u32 {
        self.timeout * 1000
    }
}
