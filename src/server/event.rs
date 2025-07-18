use crate::proto::client::Messages;
use crate::proto::common::Control;
use crate::server::Client;
use crate::server::client::ClientId;

#[expect(clippy::enum_variant_names)]
pub enum Event {
    ClientAccepted(Client),
    ClientDisconnected(ClientId),
    ClientMessages(ClientId, Messages),
    ClientControl(ClientId, Control),
}
