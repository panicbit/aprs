use std::path::PathBuf;

use crate::net::BindAddr;

#[derive(Clone)]
pub struct Config {
    pub state_path: PathBuf,
    pub bind_address: BindAddr,
}
