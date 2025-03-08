use std::collections::BTreeMap;

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
    let mut game = Game::load(cli.multiworld_path)?;

    println!("{:#?}", game.multi_data.version);

    let game_to_location_id_to_name = game
        .multi_data
        .data_package
        .iter()
        .map(|(game, package)| {
            let location_id_to_name = package
                .location_name_to_id
                .iter()
                .map(|(name, id)| (*id, name.as_str()))
                .collect::<BTreeMap<_, _>>();

            (game.clone(), location_id_to_name)
        })
        .collect::<BTreeMap<_, _>>();

    let game_to_item_id_to_name = game
        .multi_data
        .data_package
        .iter()
        .map(|(game, package)| {
            let item_id_to_name = package
                .item_name_to_id
                .iter()
                .map(|(name, id)| (*id, name.as_str()))
                .collect::<BTreeMap<_, _>>();

            (game.clone(), item_id_to_name)
        })
        .collect::<BTreeMap<_, _>>();

    for (slot, locations) in &game.multi_data.locations {
        continue;
        let game_name = &game.multi_data.slot_info[slot].game;

        println!("Slot #{} ({game_name}) checks:", slot.0);

        for (location_id, location) in locations {
            let location_id_to_name = &game_to_location_id_to_name[game_name];
            let location_name = location_id_to_name[location_id];

            let item_game_name = &game.multi_data.slot_info[&location.slot].game;
            let item_id_to_name = &game_to_item_id_to_name[item_game_name];
            let item_name = item_id_to_name[&location.item];
            let item_slot = location.slot.0;

            println!("`{location_name}` has `{item_name}` of slot #{item_slot} ({item_game_name})");
        }
    }

    let config = Config::default();
    let server = Server::new(config, game.multi_data);

    Runtime::new()?.block_on(server.run())
}
