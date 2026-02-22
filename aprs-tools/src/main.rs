use aprs_tools::Cli;
use clap::Parser;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    aprs_tools::run(Cli::parse())
}
