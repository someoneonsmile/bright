use std::path::PathBuf;

use clap::Parser;

/// bright
#[derive(Parser, Debug)]
#[clap(author, version, about, name = "bright")]
pub struct Opt {
    /// config path
    #[clap(short, long = "config")]
    pub config_file: Option<PathBuf>,
}
