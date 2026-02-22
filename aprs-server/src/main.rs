use aprs_server::Cli;
use clap::Parser;

use color_eyre::eyre::Result;

fn main() -> Result<()> {
    aprs_server::run(Cli::parse())
}
