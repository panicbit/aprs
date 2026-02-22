use color_eyre::eyre::Result;

pub mod slot_data;

#[derive(clap::Parser)]
pub enum Cli {
    SlotData(slot_data::Cli),
}

pub fn run(cli: Cli) -> Result<()> {
    let rt = aprs_utils::default_main_setup()?;

    let future = match cli {
        Cli::SlotData(cli) => slot_data::run(cli),
    };

    rt.block_on(future)
}
