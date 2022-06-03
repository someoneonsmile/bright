use chrono::{Local, NaiveTime};
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
    /// 亮度时间线
    pub time_bright: Vec<DeviceTimeItem>,
    /// 过渡方式
    pub transition: DeviceTransition,
    /// 调整亮度时的间隔, millis
    pub interval: u32,
    /// 每次间隔缓动前进百分比 1-100
    pub easing_percent: Option<u32>,
    /// 最小移动距离
    pub min_step: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub(crate) enum DeviceTransition {
    Brust,
    Line,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DeviceTimeItem {
    pub time: NaiveTime,
    pub bright: u32,
}

impl Config {
    /// parse config file
    pub(crate) fn from_toml<P: AsRef<Path>>(config_path: P) -> anyhow::Result<Option<Config>> {
        let config_path = util::shell_expend_full(config_path)?;
        println!("config path: {:?}", config_path);
        if !config_path.exists() {
            return Ok(None);
        }
        let config_str = fs::read_to_string(config_path)?;
        let mut config: Config = toml::from_str(&config_str)?;
        for (_, dev) in config.dev.iter_mut() {
            dev.time_bright.sort_by_key(|it| it.time)
        }
        Ok(Some(config))
    }
}

impl DeviceConfig {
    /// 计算亮度值
    pub(crate) fn calc_next_val(&self) -> Option<u32> {
        let pre_target = self.get_pre_target()?;
        let next_target = self.get_next_target()?;
        let next_val = self.transition.calc_next_val(
            pre_target.time,
            next_target.time,
            pre_target.bright,
            next_target.bright,
        );
        Some(next_val)
    }

    pub(crate) fn get_pre_target(&self) -> Option<&DeviceTimeItem> {
        let now = Local::now().time();
        self.time_bright
            .iter()
            .rfind(|it| it.time < now)
            .or_else(|| self.time_bright.last())
    }

    pub(crate) fn get_next_target(&self) -> Option<&DeviceTimeItem> {
        let now = Local::now().time();
        self.time_bright
            .iter()
            .find(|it| it.time > now)
            .or_else(|| self.time_bright.first())
    }

    pub(crate) fn get_target(&self) -> Option<&DeviceTimeItem> {
        match self.transition {
            DeviceTransition::Brust => self.get_pre_target(),
            DeviceTransition::Line => self.get_next_target(),
        }
    }
}

impl DeviceTransition {
    fn calc_next_val(
        &self,
        pre_target_time: NaiveTime,
        next_target_time: NaiveTime,
        pre_target_val: u32,
        next_target_val: u32,
    ) -> u32 {
        match *self {
            //
            //  | --
            //  |   |
            //  |    --
            //   --------
            Self::Brust => pre_target_val,
            //
            //  |   /
            //  |  /
            //  | /
            //   --------
            Self::Line => {
                let current_time = Local::now().time();
                (pre_target_val as i32
                    + (next_target_val as i32 - pre_target_val as i32)
                        * duration_millisec(pre_target_time, current_time)
                        / duration_millisec(pre_target_time, next_target_time))
                    as u32
            }
        }
    }
}

fn duration_millisec(from: NaiveTime, to: NaiveTime) -> i32 {
    let mut duration = to - from;
    if duration < chrono::Duration::zero() {
        duration = duration + chrono::Duration::days(1);
    }
    duration.num_milliseconds() as i32
}
