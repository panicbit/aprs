use bytes::Bytes;
use std::ops;

use crate::server::ClientMessages;

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

impl From<ClientMessages> for ControlOrMessage<ClientMessages> {
    fn from(value: ClientMessages) -> Self {
        Self::Message(value)
    }
}

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
