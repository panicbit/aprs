use bytes::Bytes;
use std::ops;
use std::sync::Arc;

pub mod network_version;
pub use network_version::NetworkVersion;

use crate::proto::{client, server};

#[derive(Debug)]
pub enum ControlOrMessage<M> {
    Control(Control),
    Message(M),
}

impl<M, C> From<C> for ControlOrMessage<M>
where
    C: Into<Control>,
{
    fn from(value: C) -> Self {
        Self::Control(value.into())
    }
}

impl From<client::Messages> for ControlOrMessage<client::Messages> {
    fn from(value: client::Messages) -> Self {
        Self::Message(value)
    }
}

#[derive(Debug)]
pub enum Control {
    Ping(Ping),
    Pong(Pong),
    Close(Close),
}

impl From<Ping> for Control {
    fn from(value: Ping) -> Self {
        Self::Ping(value)
    }
}

impl From<Pong> for Control {
    fn from(value: Pong) -> Self {
        Self::Pong(value)
    }
}

impl From<Close> for Control {
    fn from(value: Close) -> Self {
        Self::Close(value)
    }
}

#[derive(Debug, Clone)]
pub struct Ping(pub Bytes);

impl ops::Deref for Ping {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Pong(pub Bytes);

impl ops::Deref for Pong {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Close;
