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
