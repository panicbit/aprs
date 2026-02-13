use std::path::Path;

use aprs::game::Game;
use aprs::websocket_server::{Config, WebsocketServer, WebsocketServerHandle};
use tempfile::TempDir;

pub struct TestServer {
    temp_dir: TempDir,
    server_handle: WebsocketServerHandle,
    port: u16,
}

impl TestServer {
    pub async fn start(multi_world_path: impl AsRef<Path>) -> Self {
        let multi_world_path = multi_world_path.as_ref();
        let game =
            Game::load_from_zip_or_bare(multi_world_path).expect("failed to load multi world");
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let state_path = temp_dir.path().join("state.apsave");
        let config = Config {
            listen_address: ([127, 0, 0, 0], 0).into(),
            state_path,
        };
        let server_handle = WebsocketServer::new(config, game.multi_data)
            .expect("failed to start websocket server")
            .listen()
            .await
            .expect("failed to listen");
        let port = server_handle.port();

        Self {
            temp_dir,
            server_handle,
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        panic!("TODO: implement server shutdown");
    }
}
