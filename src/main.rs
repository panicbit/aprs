use std::net::Ipv4Addr;

use aprs::config::{self, Config, General};
use aprs::server::Server;
use aprs::websocket_server::{self, WebsocketServer};
use clap::Parser;
use eyre::Result;
use kameo::{Actor, mailbox};
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
        general: General {
            state_path: cli.multiworld_path.with_extension("aprs.state"),
        },
        websocket: config::WebSocket {
            listen_address: (Ipv4Addr::UNSPECIFIED, 18283).into(),
        },
    };

    Runtime::new()?.block_on(async move {
        let prepared_server = Server::prepare_with_mailbox(mailbox::bounded(10_000));
        let server = prepared_server.actor_ref().clone();

        let prepared_websocket_server = WebsocketServer::prepare();
        let websocket_server = prepared_server.actor_ref();

        server.link(websocket_server).await;

        prepared_server.spawn((config.general, game.multi_data));
        prepared_websocket_server.spawn((server.clone(), config.websocket));

        info!("Server started.");
        server.wait_for_shutdown().await;
        info!("Server stopped.");

        Ok(())
    })
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
