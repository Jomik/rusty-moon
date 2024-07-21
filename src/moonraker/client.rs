use super::api::PrinterStatusNotification;
use super::client_builder::ClientBuilder;
use anyhow::Result;
use jsonrpsee::{
    core::client::{ClientT, Subscription, SubscriptionClientT},
    ws_client::WsClient,
};
use jsonrpsee_core::params::ObjectParams;
use serde_json::json;
use tokio::sync::mpsc::Sender;

pub struct Client {
    pub(crate) client: WsClient,
}

impl Client {
    pub fn builder(url: impl AsRef<str>) -> ClientBuilder {
        ClientBuilder::new(url)
    }

    pub async fn subscribe_printer_status(
        &self,
        tx: Sender<PrinterStatusNotification>,
    ) -> Result<()> {
        let mut params = ObjectParams::new();
        params.insert(
            "objects",
            json!({
                    "display_status": ["progress", "message"],
                    "idle_timeout": ["state", "printing_time"],
                    "print_stats": ["info", "filename", "total_duration", "print_duration", "filament_used", "state", "message"],
                    "webhooks": ["state", "state_message"],
            }),
        )?;
        let response: PrinterStatusNotification = self
            .client
            .request("printer.objects.subscribe", params)
            .await?;
        tracing::info!("initial: {:?}", response);

        let mut sub: Subscription<PrinterStatusNotification> = self
            .client
            .subscribe_to_method("notify_status_update")
            .await?;
        while let Some(result) = sub.next().await {
            let notif = result?;
            tracing::info!("status update: {:?}", notif);
            tx.send(notif).await?;
        }
        Ok(())
    }
}
