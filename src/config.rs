use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::util;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    pub dev: HashMap<String, DeviceConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DeviceConfig {
    pub time_bright: Vec<DeviceConfigItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DeviceConfigItem {
    pub time: NaiveTime,
    pub bright: u32,
}

impl Config {
    /// parse config file
    pub fn from_toml<P: AsRef<Path>>(config_path: P) -> anyhow::Result<Option<Config>> {
        let config_path = util::shell_expend_full(config_path)?;
        println!("config path: {:?}", config_path);
        if !config_path.exists() {
            return Ok(None);
        }
        let config_str = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(Some(config))
    }
}
