use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, name = "bright")]
pub(crate) struct Opt {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// show the brightness of multi monitors
    Show,
    /// adjust brightness
    Run {
        /// config path
        #[clap(short, long = "config")]
        config_file: Option<PathBuf>,
    },
}
