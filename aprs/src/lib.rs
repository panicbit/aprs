use eyre::Result;

#[derive(clap::Parser)]
pub enum Cli {
    Server(aprs_server::Cli),
}

pub fn run(cli: Cli) -> Result<()> {
    match cli {
        Cli::Server(cli) => aprs_server::run(cli),
    }
}
