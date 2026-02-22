use aprs::Cli;
use clap::Parser;
use eyre::Result;

fn main() -> Result<()> {
    aprs::run(Cli::parse())
}
