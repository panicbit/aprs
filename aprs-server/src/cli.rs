use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    pub multiworld_path: PathBuf,
    #[clap(long)]
    pub only_load: bool,
}
