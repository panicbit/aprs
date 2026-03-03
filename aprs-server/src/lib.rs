#![allow(clippy::let_and_return)]

use std::hash::BuildHasherDefault;
use std::time::Instant;

use color_eyre::Result;
use color_eyre::eyre::Context;
use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use tracing::info;

use crate::config::Config;
use crate::game::Game;
use crate::net::Bind;
use crate::server::Server;

mod cli;
pub use cli::Cli;

pub mod config;
pub mod game;
pub mod net;
pub mod server;
pub mod websocket;

type Hasher = FxHasher;
type FnvIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<Hasher>>;

// TODO: put jemalloc behind a feature gate (allow reverse dependencies to opt out)
#[cfg(not(target_os = "windows"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub fn run(cli: Cli) -> Result<()> {
    let rt = aprs_utils::default_main_setup()?;

    info!("Loading world...");
    let load_start = Instant::now();
    let game = Game::load_from_zip_or_bare(&cli.multiworld_path)?;
    let load_time = load_start.elapsed();
    info!("Loading finished in {load_time:?}");

    if cli.only_load {
        return Ok(());
    }

    let config = Config {
        bind_address: cli.bind_address,
        state_path: cli.multiworld_path.with_extension("aprs.state"),
    };

    rt.block_on(start(game, config))
}

async fn start(game: Game, config: Config) -> Result<()> {
    let listener = config
        .bind_address
        .bind()
        .await
        .with_context(|| format!("failed to listen on {:?})", config.bind_address))?;

    let server = Server::new(config, game.multi_data)?;
    let server_handle = server.handle();

    websocket::start(listener, server_handle);

    info!("Server started.");

    server.run().await
}
