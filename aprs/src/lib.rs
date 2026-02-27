use color_eyre::eyre::Result;

#[derive(clap::Parser)]
pub enum Cli {
    Server(aprs_server::Cli),
    WebHost(aprs_web_host::Cli),
    #[clap(subcommand)]
    Tools(aprs_tools::Cli),
}

pub fn run(cli: Cli) -> Result<()> {
    match cli {
        Cli::Server(cli) => aprs_server::run(cli),
        Cli::WebHost(cli) => aprs_web_host::run(cli),
        Cli::Tools(cli) => aprs_tools::run(cli),
    }
}
