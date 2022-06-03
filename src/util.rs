use std::path::{Path, PathBuf};

pub(crate) fn shell_expend_full<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let origin = path
        .as_ref()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path error"))?;
    return Ok(PathBuf::from(shellexpand::full(origin)?.as_ref()));
}

pub(crate) use shellexpand::tilde;

// #[inline]
// pub(crate) fn tilde<SI>(path: &SI) -> Cow<str>
// where
//     SI: ?Sized + AsRef<str>,
// {
//     shellexpand::tilde(path)
// }

pub(crate) fn easing(current_val: i32, set_val: i32, easing_percent: u32, min_step: u32) -> i32 {
    let min_step = min_step as i32;
    let easing_set_val = match (set_val - current_val) * easing_percent as i32 / 100 {
        0 => {
            if set_val > current_val {
                (set_val).min(current_val + min_step)
            } else {
                (set_val).max(current_val - min_step)
            }
        }
        // 如果超过 set_val, 则退回到 current_val
        v if v > 0 => match current_val + v.abs().max(min_step) {
            v if v > set_val => current_val,
            _ => v,
        },
        // 如果超过 set_val, 则退回到 current_val
        v if v < 0 => match current_val - v.abs().max(min_step) {
            v if v < set_val => current_val,
            _ => v,
        },
        _ => {
            unreachable!()
        }
    };
    easing_set_val
}
