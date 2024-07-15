use jsonrpsee::{core::client::ClientT, rpc_params, ws_client::WsClientBuilder};
use serde_json::Value;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish()
        .try_init()?;

    let client = WsClientBuilder::default()
        .build("ws://localhost:7125/websocket")
        .await?;
    let response: Value = client.request("printer.info", rpc_params![]).await?;
    tracing::info!("response: {:?}", response.to_string());

    Ok(())
}
