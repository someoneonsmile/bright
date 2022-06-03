use brightness::{Brightness, BrightnessDevice};
use chrono::naive::NaiveTime;
use chrono::Local;
use config::DeviceConfig;
use futures::{StreamExt, TryStreamExt};
use std::collections::HashMap;
use tokio::time::{self, MissedTickBehavior};

mod config;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // get device info
    let dev_map = get_dev_map().await?;

    // get config
    let config_path = "${XDG_CONFIG_HOME:-~/.config}/bright/config.toml";
    let config = config::Config::from_toml(config_path)?.ok_or_else(|| {
        anyhow::anyhow!(format!("can't not find the config file {}", config_path))
    })?;

    // show current value for each device
    show_brightnes(&dev_map).await?;

    // concurrent adjust the brightness
    futures::stream::iter(dev_map.into_iter())
        .map(Ok)
        .try_for_each_concurrent(None, |(dev_name, dev)| async {
            match config.dev.get(&dev_name) {
                Some(dev_config) => {
                    set_brightnes(dev_name, dev, dev_config).await?;
                }
                _ => {
                    eprintln!("can't find config for {}", dev_name);
                }
            }
            Ok(()) as anyhow::Result<()>
        })
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
    let mut interval = time::interval(std::time::Duration::from_millis(200));
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
                    let mut sleep_duration =
                        next_wake_time.signed_duration_since(Local::now().time());
                    if sleep_duration < chrono::Duration::zero() {
                        sleep_duration = sleep_duration + chrono::Duration::days(1);
                    }
                    println!("sleep: {:?}, wake at {}", sleep_duration, next_wake_time);
                    time::sleep(sleep_duration.to_std()?).await;
                }
                _ => {
                    let brightness = calc_next_bright(cur_value, target);
                    println!(
                        "Brightness of device {}, current={}%, target={}%, set={}%",
                        dev_name, cur_value, target, brightness
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
        .rfind(|it| it.time < now)
        .or_else(|| dev_config.time_bright.last())
        .map(|it| it.bright)
}

fn get_next_wake_time(dev_config: &DeviceConfig) -> anyhow::Result<NaiveTime> {
    let now = Local::now().time();
    dev_config
        .time_bright
        .iter()
        .find(|it| it.time > now)
        .or_else(|| dev_config.time_bright.first())
        .map(|it| it.time)
        .ok_or_else(|| anyhow::anyhow!("not find the next wake time"))
}

fn calc_next_bright(cur_value: u32, target: u32) -> u32 {
    // TODO: * 10 read from config
    match (target as i32 - cur_value as i32) * 10 / 100 {
        0 => (cur_value as i32 + if target > cur_value { 1 } else { -1 }) as u32,
        v => cur_value + v as u32,
    }
}
