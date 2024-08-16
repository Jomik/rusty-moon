use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

use anyhow::Result;
use jsonrpsee::ws_client::WsClientBuilder;

use super::{client::Client, Config};

pub struct ClientBuilder {
    host: String,
    port: Option<u16>,
}

impl ClientBuilder {
    pub fn new(config: Config) -> ClientBuilder {
        ClientBuilder {
            host: config.host,
            port: config.port,
        }
    }
}

impl IntoFuture for ClientBuilder {
    type Output = Result<Client>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let client = WsClientBuilder::default()
                .build(format!(
                    "ws://{}:{}/websocket",
                    self.host,
                    self.port.unwrap_or(7125)
                ))
                .await?;

            Ok(Client {
                client,
                host: self.host,
            })
        })
    }
}
