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
    let config_path = format!(
        "${{XDG_CONFIG_HOME:-{}}}/bright/config.toml",
        util::tilde("~/.config")
    );
    let config = config::Config::from_toml(&config_path)?.ok_or_else(|| {
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
    let mut interval = time::interval(std::time::Duration::from_millis(dev_config.interval as u64));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let target = dev_config.get_target().map(|it| it.bright);
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
                    let brightness = dev_config
                        .calc_next_val(cur_value)
                        .ok_or_else(|| anyhow::anyhow!("can't calc the next bright"))?;
                    println!(
                        "Brightness of device {}, current={}%, target={}% set={}%",
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

#[inline]
fn get_next_wake_time(dev_config: &DeviceConfig) -> anyhow::Result<NaiveTime> {
    dev_config
        .get_next_target()
        .map(|it| it.time)
        .ok_or_else(|| anyhow::anyhow!("can't find the next wake time"))
}
