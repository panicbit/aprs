use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;

use color_eyre::eyre::{Context, Result, bail};
use tokio::net::UnixListener;
use tokio::{fs, io};

use crate::net::Bind;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct UnixAddr(PathBuf);

impl Bind for UnixAddr {
    type Listener = UnixListener;

    async fn bind(&self) -> Result<Self::Listener> {
        // NOTE: UnixListener::bind is not async for some reason?

        match fs::metadata(&self.0).await {
            Err(err) => {
                if err.kind() != io::ErrorKind::NotFound {
                    bail!(err)
                }
            }
            Ok(metadata) => {
                if metadata.file_type().is_socket() {
                    fs::remove_file(&self.0)
                        .await
                        .context("failed to remove existing socket")?;
                }
            }
        }

        Ok(UnixListener::bind(&self.0)?)
    }
}

impl From<PathBuf> for UnixAddr {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}

impl From<&str> for UnixAddr {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for UnixAddr {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}
