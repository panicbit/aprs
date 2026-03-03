use std::path::PathBuf;

use clap::Parser;

use crate::net::BindAddr;

#[derive(Parser)]
pub struct Cli {
    pub multiworld_path: PathBuf,
    #[clap(long = "bind", default_value = "0.0.0.0:18283")]
    pub bind_address: BindAddr,
    #[clap(long)]
    pub only_load: bool,
}
