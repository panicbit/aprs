use aprs_client::Client;
use aprs_proto::client::{Connect, ItemsHandling};
use aprs_proto::primitives::ConnectName;
use color_eyre::eyre::{Context, Result};

#[derive(clap::Parser)]
pub struct Cli {
    pub addr: String,
    pub player: String,
    #[clap(long("pw"))]
    pub password: Option<String>,
}

pub async fn run(cli: Cli) -> Result<()> {
    let Cli {
        addr,
        player,
        password,
    } = cli;
    let mut client = Client::connect(addr)
        .await
        .context("failed to connect to server")?;

    let connect = Connect {
        password,
        game: "".into(),
        name: ConnectName(player),
        uuid: "".into(),
        version: (1, 6, 6).into(),
        items_handling: ItemsHandling::empty(),
        tags: vec!["AP".into(), "TextOnly".into()],
        slot_data: true,
    };
    let connected = client.login(connect).await.context("failed to log in")?;

    let slot_info = serde_json::to_string_pretty(&connected.slot_info)
        .context("failed to serialize slot info")?;

    println!("{slot_info}");

    Ok(())
}
