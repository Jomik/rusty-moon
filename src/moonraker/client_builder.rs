use anyhow::Result;
use jsonrpsee::ws_client::WsClientBuilder;

use super::client::Client;

pub struct ClientBuilder {}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder {}
    }
    pub async fn build(self, url: String) -> Result<Client> {
        let client = WsClientBuilder::default().build(url).await?;

        Ok(Client { client })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}
