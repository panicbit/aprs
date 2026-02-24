use color_eyre::eyre::Result;

pub mod slot_data;
pub mod slot_info;

#[derive(clap::Parser)]
pub enum Cli {
    SlotData(slot_data::Cli),
    SlotInfo(slot_info::Cli),
}

pub fn run(cli: Cli) -> Result<()> {
    let rt = aprs_utils::default_main_setup()?;

    match cli {
        Cli::SlotData(cli) => rt.block_on(slot_data::run(cli)),
        Cli::SlotInfo(cli) => rt.block_on(slot_info::run(cli)),
    }
}
