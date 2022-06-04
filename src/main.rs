use brightness::{Brightness, BrightnessDevice};
use chrono::naive::NaiveTime;
use chrono::Local;
use clap::Parser;
use config::DeviceConfig;
use futures::{StreamExt, TryStreamExt};
use std::collections::HashMap;
use tokio::time::{self, MissedTickBehavior};

mod cli;
mod config;
mod util;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let opt = cli::Opt::parse();

    // get device info
    let dev_map = get_dev_map().await?;

    // get config
    let config_path = opt
        .config_file
        .map(|it| it.to_string_lossy().to_string())
        .unwrap_or(format!(
            "${{XDG_CONFIG_HOME:-{}}}/bright/config.toml",
            util::tilde("~/.config")
        ));
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
        if let Some(target) = target {
            let cur_value = dev.get().await?;
            match cur_value {
                cur_value if cur_value == target => {
                    sleep_until_next(dev_config).await?;
                }
                _ => {
                    easing_set(&mut dev, &dev_name, dev_config, cur_value, target).await?;
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

async fn sleep_until_next(dev_config: &DeviceConfig) -> anyhow::Result<()> {
    let next_wake_time = get_next_wake_time(dev_config)?;
    let mut sleep_duration = next_wake_time.signed_duration_since(Local::now().time());
    if sleep_duration < chrono::Duration::zero() {
        sleep_duration = sleep_duration + chrono::Duration::days(1);
    }
    println!("sleep: {:?}, wake at {}", sleep_duration, next_wake_time);
    time::sleep(sleep_duration.to_std()?).await;
    Ok(())
}

async fn easing_set(
    dev: &mut BrightnessDevice,
    dev_name: &str,
    dev_config: &DeviceConfig,
    current_val: u32,
    target_val: u32,
) -> anyhow::Result<()> {
    // 根据 transition 计算亮度值
    let set_val = dev_config
        .calc_next_val()
        .ok_or_else(|| anyhow::anyhow!("can't calc the next bright"))?;

    // 计算缓动值
    let easing_set_val = util::easing(
        current_val as i32,
        set_val as i32,
        dev_config.easing_percent.unwrap_or(100),
        dev_config.min_step.unwrap_or(1),
    ) as u32;

    println!(
        "Brightness of device {}, current={}%, target={}% set={}%, easing_set={}%",
        dev_name, current_val, target_val, set_val, easing_set_val
    );

    // 判断是否有必要调用 set
    if current_val != easing_set_val {
        dev.set(easing_set_val).await?;
    }
    Ok(())
}
