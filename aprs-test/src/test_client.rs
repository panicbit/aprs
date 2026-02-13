use aprs_proto::client::Connect;

use crate::TestServer;

pub struct TestClient {}

impl TestClient {
    pub fn connect(test_server: &TestServer, connect: Connect) -> Self {}
}
