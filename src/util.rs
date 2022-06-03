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
    let easing_percent = easing_percent as i32;
    let easing_set_val = match (set_val - current_val) * easing_percent / 100 {
        0 => {
            current_val + (set_val - current_val).signum() * min_step
            // if set_val > current_val {
            //     current_val + min_step
            // } else {
            //     current_val - min_step
            // }
        }
        v => current_val + v.signum() * v.abs().max(min_step),
    };
    if (easing_set_val - current_val).signum() * (easing_set_val - set_val).signum() > 0 {
        current_val
    } else {
        easing_set_val
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_easing() {
        // assert_eq!(-80 * 10 / 100, 0);
        let r = easing(21, 20, 10, 2);
        assert_eq!(r, 21);
    }
}
