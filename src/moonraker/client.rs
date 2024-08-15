use super::api::{IdentifyResult, PrinterObjectStatusResponse, ServerInfoResponse};
use super::client_builder::ClientBuilder;
use anyhow::Result;
use jsonrpsee::{
    core::{
        client::{ClientT, Subscription, SubscriptionClientT},
        params::ObjectParams,
    },
    rpc_params,
    ws_client::WsClient,
};
use serde_json::json;

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

    pub async fn get_server_info(&self) -> Result<ServerInfoResponse> {
        let response = self.client.request("server.info", rpc_params![]).await?;

        Ok(response)
    }

    pub async fn get_printer_status(&self) -> Result<PrinterObjectStatusResponse> {
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
        let response = self.client.request("printer.objects.query", params).await?;

        Ok(response)
    }

    pub async fn register_remote_method(&self, method: impl AsRef<str>) -> Result<()> {
        let mut params = ObjectParams::new();
        let method = method.as_ref();
        params.insert("method_name", method)?;
        let response: String = self
            .client
            .request("connection.register_remote_method", params)
            .await?;
        tracing::debug!("register_remote_method({:?}): {:?}", method, response);

        Ok(())
    }

    pub async fn register_printer_subscription(&self) -> Result<PrinterObjectStatusResponse> {
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
        let response: PrinterObjectStatusResponse = self
            .client
            .request("printer.objects.subscribe", params)
            .await?;
        Ok(response)
    }

    pub async fn subscribe_remote_method<Params>(
        &self,
        method: impl AsRef<str>,
    ) -> Result<Subscription<Params>>
    where
        Params: serde::de::DeserializeOwned,
    {
        let method = method.as_ref();
        let sub = self.client.subscribe_to_method(method).await?;
        Ok(sub)
    }

    pub async fn subscribe_printer_status(
        &self,
    ) -> Result<Subscription<PrinterObjectStatusResponse>> {
        let sub: Subscription<PrinterObjectStatusResponse> = self
            .client
            .subscribe_to_method("notify_status_update")
            .await?;

        Ok(sub)
    }

    pub async fn subscribe_klippy_ready(&self) -> Result<Subscription<()>> {
        let sub: Subscription<()> = self
            .client
            .subscribe_to_method("notify_klippy_ready")
            .await?;
        Ok(sub)
    }

    pub async fn subscribe_klippy_shutdown(&self) -> Result<Subscription<()>> {
        let sub: Subscription<()> = self
            .client
            .subscribe_to_method("notify_klippy_shutdown")
            .await?;
        Ok(sub)
    }

    pub async fn subscribe_klippy_disconnected(&self) -> Result<Subscription<()>> {
        let sub: Subscription<()> = self
            .client
            .subscribe_to_method("notify_klippy_disconnected")
            .await?;
        Ok(sub)
    }
}
