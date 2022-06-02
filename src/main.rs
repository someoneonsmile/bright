use brightness::{Brightness, BrightnessDevice};
use chrono::naive::NaiveTime;
use chrono::Local;
use config::DeviceConfig;
use futures::{StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{self, MissedTickBehavior};

mod config;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let dev_map = get_dev_map().await?;
    let config_path = "${XDG_CONFIG:~/.config}/bright/config.toml";
    let config = config::Config::from_toml(config_path)?.ok_or_else(|| {
        anyhow::anyhow!(format!("can't not find the config file {}", config_path))
    })?;

    // show current value for each device
    show_brightnes(&dev_map).await?;

    // concurrent adjust the brightness
    futures::stream::iter(dev_map)
        .then(|(dev_name, dev)| async {
            let dev_config = config.dev.get(&dev_name).ok_or_else(|| {
                anyhow::anyhow!(format!("can't find the config for device: {}", dev_name))
            })?;
            set_brightnes(dev_name, dev, dev_config).await?;
            Ok(()) as anyhow::Result<()>
        })
        .try_collect()
        .await?;
    Ok(())
}

async fn get_dev_map() -> anyhow::Result<HashMap<String, BrightnessDevice>> {
    let r = brightness::brightness_devices()
        .try_fold(HashMap::new(), |mut acc, it| async {
            acc.insert(it.device_name().await?, it);
            Ok(acc)
        })
        .await?;
    Ok(r)
}

// TODO: args config
async fn set_brightnes(
    dev_name: String,
    mut dev: BrightnessDevice,
    dev_config: &config::DeviceConfig,
) -> anyhow::Result<()> {
    let mut interval = time::interval(Duration::from_millis(200));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let target = get_target_brightness(dev_config);
        // set_brightnes().await?;
        if let Some(target) = target {
            let cur_value = dev.get().await?;
            match cur_value {
                cur_value if cur_value == target => {
                    let next_wake_time = get_next_wake_time(dev_config)?;
                    let sleep_duration = next_wake_time
                        .signed_duration_since(Local::now().time())
                        .to_std()?;
                    println!("sleep: {:?}, wake at {}", sleep_duration, next_wake_time);
                    time::sleep(sleep_duration).await;
                }
                _ => {
                    // TODO: * 10 read from config
                    let brightness = match (target as i32 - cur_value as i32) * 10 / 100 {
                        0 => (cur_value as i32 + if target > cur_value { 1 } else { -1 }) as u32,
                        v => cur_value + v as u32,
                    };
                    println!(
                        "Brightness of device {}, target={}%, set={}%",
                        dev_name, target, brightness
                    );
                    dev.set(brightness).await?;
                }
            };
        }
    }
}

async fn show_brightnes(dev_map: &HashMap<String, BrightnessDevice>) -> anyhow::Result<()> {
    for (dev_name, dev) in dev_map.iter() {
        let value = dev.get().await?;
        println!("Brightness of device {} is {}%", dev_name, value);
    }
    Ok(())
}

fn get_target_brightness(dev_config: &DeviceConfig) -> Option<u32> {
    let now = Local::now().time();
    dev_config
        .time_bright
        .iter()
        .rfind(|it| it.0 < now)
        .or_else(|| dev_config.time_bright.last())
        .map(|it| it.1)
}

fn get_next_wake_time(dev_config: &DeviceConfig) -> anyhow::Result<NaiveTime> {
    let now = Local::now().time();
    dev_config
        .time_bright
        .iter()
        .find(|it| it.0 > now)
        .or_else(|| dev_config.time_bright.first())
        .map(|it| it.0)
        .ok_or_else(|| anyhow::anyhow!("not find the next wake time"))
}
