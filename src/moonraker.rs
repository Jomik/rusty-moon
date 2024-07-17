use std::borrow::Cow;

use tokio::sync::watch::Receiver;

pub use self::api::*;

mod api;
mod client;
mod client_builder;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub url: Cow<'static, str>,
}

pub async fn connect(conf: Config) -> anyhow::Result<Receiver<PrinterStatusNotification>> {
    let client = client::Client::builder()
        .build(conf.url.into_owned())
        .await?;
    let response = client.request_server_info().await?;
    tracing::info!("server: {:?}", response.to_string());
    let response = client.request_printer_info().await?;
    tracing::info!("printer: {:?}", response.to_string());

    let rc = client.watch_printer_status().await?;

    Ok(rc)
}
