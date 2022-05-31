use brightness::Brightness;
use chrono::naive::NaiveTime;
use chrono::Local;
use futures::TryStreamExt;
use std::time::Duration;
use std::{collections::HashMap, ops::RangeBounds};
use tokio::time::{self, MissedTickBehavior};

use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG: HashMap<std::ops::Range<NaiveTime>, u32> = {
        let mut m = HashMap::new();
        m.insert(
            NaiveTime::from_hms(0, 0, 0)..NaiveTime::from_hms(8, 0, 0),
            20,
        );
        m.insert(
            NaiveTime::from_hms(8, 0, 0)..NaiveTime::from_hms(10, 0, 0),
            30,
        );
        m.insert(
            NaiveTime::from_hms(8, 0, 0)..NaiveTime::from_hms(10, 0, 0),
            50,
        );
        m.insert(
            NaiveTime::from_hms(10, 0, 0)..NaiveTime::from_hms(18, 0, 0),
            80,
        );
        m.insert(
            NaiveTime::from_hms(18, 0, 0)..NaiveTime::from_hms(19, 0, 0),
            70,
        );
        m.insert(
            NaiveTime::from_hms(19, 0, 0)..NaiveTime::from_hms(20, 0, 0),
            60,
        );
        m.insert(
            NaiveTime::from_hms(20, 0, 0)..NaiveTime::from_hms(23, 59, 59),
            20,
        );
        m
    };
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    show_brightnes().await?;
    let mut interval = time::interval(Duration::from_millis(200));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        set_brightnes().await?;
    }
}

fn find_target() -> Option<u32> {
    let now = Local::now().time();
    CONFIG
        .iter()
        .find(|(k, _v)| k.contains(&now))
        .map(|(_k, v)| *v)
}

fn find_next_wake_time() -> Option<NaiveTime> {
    let now = Local::now().time();
    CONFIG
        .iter()
        .find(|(k, _v)| k.contains(&now))
        .and_then(|(k, _v)| match k.end_bound() {
            std::ops::Bound::Excluded(end_bound) => Some(*end_bound),
            _ => None,
        })
}

async fn set_brightnes() -> anyhow::Result<()> {
    let target = find_target();
    match target {
        Some(target) => {
            brightness::brightness_devices()
                .map_err(|e| anyhow::Error::from(e))
                .try_for_each(|mut dev| async move {
                    let name = dev.device_name().await?;
                    let cur_value = dev.get().await?;
                    println!(
                        "Brightness of device {} is {}%, target={}%",
                        name, cur_value, target
                    );
                    match cur_value {
                        v if v == target => {
                            let next_wake_time = find_next_wake_time();
                            if let Some(next_wake_time) = next_wake_time {
                                let sleep_duration =
                                    next_wake_time.signed_duration_since(Local::now().time());
                                time::sleep(sleep_duration.to_std()?).await;
                            }
                        }
                        _ => {
                            let target = match (target as i32 - cur_value as i32) / 10 {
                                0 => {
                                    (cur_value as i32 + if target > cur_value { 1 } else { -1 })
                                        as u32
                                }
                                v => cur_value + v as u32,
                            };
                            println!("Brightness of device {} set target={}%", name, target);
                            dev.set(target).await?;
                        }
                    };
                    Ok(())
                })
                .await?;
        }
        None => {}
    };

    Ok(())
}

async fn show_brightnes() -> anyhow::Result<()> {
    brightness::brightness_devices()
        .try_for_each(|dev| async move {
            let name = dev.device_name().await?;
            let value = dev.get().await?;
            println!("Brightness of device {} is {}%", name, value);
            Ok(())
        })
        .await?;
    Ok(())
}
