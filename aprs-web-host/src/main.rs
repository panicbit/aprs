use aprs_web_host::Cli;
use clap::Parser;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    aprs_web_host::run(Cli::parse())
}
