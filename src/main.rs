use std::net::Ipv4Addr;

use aprs::server::{Config, Server};
use clap::Parser;
use eyre::Result;
use tokio::runtime::Runtime;
use tracing::level_filters::LevelFilter;
use tracing::{error, info};
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::cli::Cli;
use aprs::game::Game;

mod cli;

// checksum:
// https://github.com/ArchipelagoMW/Archipelago/blob/cd761db17035254559306f835c80f91c11e3b7af/worlds/AutoWorld.py#L588

fn main() {
    color_eyre::install().unwrap();
    configure_tracing();

    if let Err(err) = run() {
        error!("{:?}", err);
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let game = Game::load(&cli.multiworld_path)?;
    let config = Config {
        listen_address: (Ipv4Addr::UNSPECIFIED, 18283).into(),
        state_path: cli.multiworld_path.with_extension("aprs.state"),
    };

    info!("Server started.");

    let server = Server::new(config, game.multi_data)?;

    Runtime::new()?.block_on(server.run())
}

fn configure_tracing() {
    tracing_subscriber::registry()
        .with(LevelFilter::DEBUG)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time(),
        )
        .with(ErrorLayer::default())
        .init();
}
