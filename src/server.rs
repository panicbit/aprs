use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use archipelago_core::game::MultiData;
use fnv::FnvHashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::tungstenite::handshake::server::Callback;
use tokio_tungstenite::tungstenite::http::Uri;

use crate::server::client::Client;
use crate::server::event::Event;

pub use config::Config;

mod client;
mod config;
mod event;
mod event_handlers;

pub struct Server {
    config: Config,
    multi_data: MultiData,
    rx: Receiver<Event>,
    tx: Sender<Event>,
    clients: FnvHashMap<SocketAddr, Arc<Mutex<Client>>>,
}

impl Server {
    pub fn new(config: Config, multi_data: MultiData) -> Self {
        let (tx, rx) = mpsc::channel(10_000);

        Self {
            config,
            multi_data,
            tx,
            rx,
            clients: FnvHashMap::default(),
        }
    }

    pub async fn run(self) -> Result<()> {
        let listen_address = self.config.listen_address;
        let listener = TcpListener::bind(listen_address).await?;

        tokio::spawn(acceptor_loop(listener, self.tx.clone()));

        self.event_loop().await;

        Ok(())
    }

    pub async fn event_loop(mut self) {
        loop {
            let Some(event) = self.rx.recv().await else {
                eprintln!("Event channel closed.");
                return;
            };

            self.on_event(event).await;
        }
    }
}

async fn acceptor_loop(listener: TcpListener, event_tx: Sender<Event>) {
    loop {
        select! {
            _ = event_tx.closed() => {
                eprintln!("acceptor loop shutting down");
                return
            },
            accepted = listener.accept() => {
                let (stream, address) = match accepted {
                    Ok(client) => client,
                    Err(err) => {
                        eprintln!("Error accepting client: {err:?}");
                        continue;
                    }
                };

                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    if let Err(err) = handle_accept(stream, address, event_tx).await {
                        eprintln!("Failed to accept client {address}: {err:?}");
                    }
                });
            }
        }
    }
}

async fn handle_accept(
    stream: TcpStream,
    address: SocketAddr,
    event_tx: Sender<Event>,
) -> Result<()> {
    eprintln!("||| {address:?} connected");

    #[derive(Default)]
    struct Data {
        uri: Uri,
    }

    impl Callback for &mut Data {
        fn on_request(
            self,
            request: &tokio_tungstenite::tungstenite::handshake::server::Request,
            response: tokio_tungstenite::tungstenite::handshake::server::Response,
        ) -> std::result::Result<
            tokio_tungstenite::tungstenite::handshake::server::Response,
            tokio_tungstenite::tungstenite::handshake::server::ErrorResponse,
        > {
            self.uri = request.uri().clone();

            Ok(response)
        }
    }

    let mut data = Data::default();
    let stream = tokio_tungstenite::accept_hdr_async(stream, &mut data).await?;

    if event_tx
        .send(Event::ClientAccepted(address, stream))
        .await
        .is_err()
    {
        eprintln!("Can't accept client, event channel is closed");
    }

    Ok(())
}
