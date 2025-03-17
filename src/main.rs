use anyhow::Result;
use aprs::server::{Config, Server};
use clap::Parser;
use tokio::runtime::Runtime;

use crate::cli::Cli;
use aprs::game::Game;

mod cli;

// checksum:
// https://github.com/ArchipelagoMW/Archipelago/blob/cd761db17035254559306f835c80f91c11e3b7af/worlds/AutoWorld.py#L588

fn main() -> Result<()> {
    let cli = Cli::parse();
    let game = Game::load(cli.multiworld_path)?;
    let config = Config::default();

    eprintln!("Server started.");
    eprintln!("{:#?}", game.multi_data.rest);

    let server = Server::new(config, game.multi_data);

    Runtime::new()?.block_on(server.run())
}
