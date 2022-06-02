use std::path::{Path, PathBuf};

// pub(crate) fn get_home_dir() -> PathBuf {
//     let home = env::var("HOME").unwrap();
//     PathBuf::from(home)
// }

// pub(crate) fn shell_expend_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
//     if path.as_ref().starts_with("~/") {
//         let mut home = get_home_dir();
//         home.push(path.as_ref().strip_prefix("~/").unwrap());
//         return home;
//     }
//     PathBuf::from(path.as_ref())
// }

pub(crate) fn shell_expend_full<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let origin = path
        .as_ref()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("path error"))?;
    return Ok(PathBuf::from(shellexpand::full(origin)?.as_ref()));
}
