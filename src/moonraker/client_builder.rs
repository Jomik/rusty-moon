use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

use anyhow::Result;
use jsonrpsee::ws_client::WsClientBuilder;

use super::client::Client;

pub struct ClientBuilder {
    url: String,
}

impl ClientBuilder {
    pub fn new(url: impl AsRef<str>) -> ClientBuilder {
        ClientBuilder {
            url: url.as_ref().to_string(),
        }
    }
}

impl IntoFuture for ClientBuilder {
    type Output = Result<Client>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let client = WsClientBuilder::default().build(self.url).await?;
            Ok(Client { client })
        })
    }
}
