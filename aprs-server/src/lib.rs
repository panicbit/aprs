#![allow(clippy::let_and_return)]

use std::hash::BuildHasherDefault;
use std::net::Ipv4Addr;
use std::time::Instant;

use color_eyre::Result;
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use tokio::runtime::Runtime;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::game::Game;
use crate::websocket_server::{Config, WebsocketServer};

mod cli;
pub use cli::Cli;

pub mod game;
pub mod server;
pub mod websocket_server;

type Hasher = FxHasher;
type FnvIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<Hasher>>;

// TODO: put jemalloc behind a feature gate (allow reverse dependencies to opt out)
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub fn run(cli: Cli) -> Result<()> {
    color_eyre::install().unwrap();
    configure_tracing();

    info!("Loading world...");
    let load_start = Instant::now();
    let game = Game::load_from_zip_or_bare(&cli.multiworld_path)?;
    let load_time = load_start.elapsed();
    info!("Loading finished in {load_time:?}");

    if cli.only_load {
        return Ok(());
    }

    let config = Config {
        listen_address: (Ipv4Addr::UNSPECIFIED, 18283).into(),
        state_path: cli.multiworld_path.with_extension("aprs.state"),
    };

    info!("Server started.");

    let server = WebsocketServer::new(config, game.multi_data)?;

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
