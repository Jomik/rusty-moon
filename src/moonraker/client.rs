use super::api::PrinterStatusNotification;
use super::client_builder::ClientBuilder;
use anyhow::Result;
use jsonrpsee::{
    core::client::{ClientT, Subscription, SubscriptionClientT},
    rpc_params,
    ws_client::WsClient,
};
use jsonrpsee_core::params::ObjectParams;
use serde_json::{json, Value as JsonValue};
use tokio::{
    spawn,
    sync::watch::{self, Receiver},
};

pub struct Client {
    pub(crate) client: WsClient,
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

    pub async fn watch_printer_status(&self) -> Result<Receiver<PrinterStatusNotification>> {
        let mut params = ObjectParams::new();
        params.insert(
            "objects",
            json!({
                    "display_status": ["progress", "message"],
                    "print_stats": ["info", "filename", "total_duration", "print_duration", "filament_used", "state", "message"],
                    "webhooks": ["state", "state_message"],
            }),
        )?;
        let response: PrinterStatusNotification = self
            .client
            .request("printer.objects.subscribe", params)
            .await?;
        tracing::info!("initial: {:?}", response);

        let (tx, rx) = watch::channel(response);

        let mut sub: Subscription<PrinterStatusNotification> = self
            .client
            .subscribe_to_method("notify_status_update")
            .await?;
        spawn(async move {
            loop {
                // Ignore subscription errors
                if let Some(Ok(response)) = sub.next().await {
                    tracing::info!("status update: {:?}", response);
                    tx.send(response).unwrap();
                }
            }
        });
        Ok(rx)
    }
}
