use color_eyre::eyre::Result;

use crate::net::Accept;

pub trait Bind {
    type Listener: Accept;

    fn bind(&self) -> impl Future<Output = Result<Self::Listener>>;
}
