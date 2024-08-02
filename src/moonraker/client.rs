use crate::moonraker::IdentifyResult;

use super::api::PrinterStatusNotification;
use super::client_builder::ClientBuilder;
use anyhow::Result;
use jsonrpsee::{
    core::{
        client::{ClientT, Subscription, SubscriptionClientT},
        params::ObjectParams,
    },
    ws_client::WsClient,
};
use serde_json::{json, Value};
use tokio::sync::watch::Sender;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const URL: &str = env!("CARGO_PKG_HOMEPAGE");

pub struct Client {
    pub(crate) client: WsClient,
}

impl Client {
    pub fn builder(url: impl AsRef<str>) -> ClientBuilder {
        ClientBuilder::new(url)
    }

    pub async fn identify(&self) -> Result<()> {
        let mut params = ObjectParams::new();
        params.insert("client_name", NAME)?;
        params.insert("version", VERSION)?;
        params.insert("url", URL)?;
        params.insert("type", "agent")?;
        let response: IdentifyResult = self
            .client
            .request("server.connection.identify", params)
            .await?;
        tracing::debug!("identify: {:?}", response);
        Ok(())
    }

    pub async fn subscribe_remote_method(&self, method: impl AsRef<str>) -> Result<()> {
        let mut params = ObjectParams::new();
        let method = method.as_ref();
        params.insert("method_name", method)?;
        let response: String = self
            .client
            .request("connection.register_remote_method", params)
            .await?;
        tracing::debug!("register_remote_method: {:?}", response);

        let mut sub: Subscription<Value> = self.client.subscribe_to_method(method).await?;
        while let Some(result) = sub.next().await {
            let notif = result?;
            tracing::trace!("{}: {:?}", method, notif);
            // tx.send(notif).await?;
        }

        Ok(())
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
        tracing::debug!("initial: {:?}", response);

        let mut sub: Subscription<PrinterStatusNotification> = self
            .client
            .subscribe_to_method("notify_status_update")
            .await?;
        while let Some(result) = sub.next().await {
            let notif = result?;
            tracing::trace!("status update: {:?}", notif);
            tx.send_replace(notif);
        }
        Ok(())
    }
}
