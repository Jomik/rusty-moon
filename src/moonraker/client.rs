use super::client_builder::ClientBuilder;
use anyhow::Result;
use jsonrpsee::{
    core::client::{ClientT, Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClient,
};
use jsonrpsee_core::params::ObjectParams;
use serde_json::{json, Value as JsonValue};

pub struct Client {
    pub(crate) client: WsClient,
}

#[derive(Debug, serde::Deserialize)]
struct PrintStatsInfo {
    current_layer: Option<u64>,
    #[allow(dead_code)]
    total_layer: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
struct PrintStats {
    info: PrintStatsInfo,
}

#[derive(Debug, serde::Deserialize)]
struct PrinterStatus {
    print_stats: PrintStats,
}
#[derive(Debug, serde::Deserialize)]
struct PrinterStatusNotification {
    status: PrinterStatus,
    #[serde(rename = "eventtime")]
    #[allow(dead_code)]
    event_time: f64,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub async fn request_printer_info(&self) -> Result<JsonValue> {
        let response: JsonValue = self.client.request("printer.info", rpc_params![]).await?;
        Ok(response)
    }

    pub async fn request_server_info(&self) -> Result<JsonValue> {
        let response: JsonValue = self.client.request("server.info", rpc_params![]).await?;
        Ok(response)
    }

    pub async fn subscribe(&self) -> Result<()> {
        let mut params = ObjectParams::new();
        params.insert(
            "objects",
            json!({
                    "print_stats": ["info"]
            }),
        )?;
        let response: PrinterStatusNotification = self
            .client
            .request("printer.objects.subscribe", params)
            .await?;
        tracing::info!("request: {:?}", response);
        let mut sub: Subscription<PrinterStatusNotification> = self
            .client
            .subscribe_to_method("notify_status_update")
            .await?;
        if let Some(notif) = sub.next().await {
            tracing::info!("notification: {:?}", notif)
        } else {
            tracing::info!("No notification")
        }
        sub.unsubscribe().await?;
        Ok(())
    }
}
