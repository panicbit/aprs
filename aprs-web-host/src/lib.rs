#[macro_use]
extern crate rocket;

use color_eyre::eyre::{Context, Result};
use rocket::data::{Limits, ToByteUnit};
use routes::routes;

mod cli;
pub use cli::Cli;

mod routes;

pub mod db;
pub mod models;
pub mod schema;

pub fn run(cli: Cli) -> Result<()> {
    let Cli {} = cli;

    db::run_pending_migrations()?;

    rocket::execute(async move {
        let limits = Limits::new()
            .limit("file", 8.mebibytes())
            .limit("data-form", 8.mebibytes());
        let figment = rocket::Config::figment().merge(("limits", limits));
        let pool = db::pool().await.context("failed to created db pool")?;

        rocket::Rocket::custom(figment)
            .mount("/", routes())
            .manage(pool)
            .launch()
            .await?;

        Ok(())
    })
}
