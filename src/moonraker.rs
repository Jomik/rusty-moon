use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use jsonrpsee::core::client::Subscription;
use serde_json::Value;
use tokio::select;
use tokio::sync::watch;

pub use self::api::*;
pub use self::status::*;

mod api;
mod client;
mod client_builder;
mod status;

const NOTIFICATION_METHOD: &str = "rusty_moon_notification";

#[derive(Clone, Debug, Default)]
pub enum KlippyState {
    #[default]
    Disconnected,
    Ready,
    Shutdown,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub url: String,
}

pub struct ServiceBuilder {
    config: Config,
}

impl ServiceBuilder {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl IntoFuture for ServiceBuilder {
    type Output = Result<Service>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let client = Arc::new(client::Client::builder(self.config.url).await?);

            Ok(Service { client })
        })
    }
}

pub struct Service {
    client: Arc<client::Client>,
}

impl Service {
    pub fn builder(config: Config) -> ServiceBuilder {
        ServiceBuilder::new(config)
    }

    pub async fn start(self, status_tx: watch::Sender<Status>) -> Result<()> {
        let client = self.client.clone();

        client.identify().await?;

        let _notification_sub: Subscription<Value> =
            client.subscribe_remote_method(NOTIFICATION_METHOD).await?;
        let mut status_sub = client.subscribe_printer_status().await?;
        let mut ready_sub = self.client.subscribe_klippy_ready().await?;
        let mut disconnected_sub = self.client.subscribe_klippy_disconnected().await?;
        let mut shutdown_sub = self.client.subscribe_klippy_shutdown().await?;

        self.update_klippy_status(self.get_initial_klippy_state().await?, &status_tx)
            .await?;
        loop {
            select! {
                // TODO: Handle errors
                Some(Ok(notif)) = status_sub.next() => {
                    status_tx.send_replace(Status::from(&notif.status));
                }
                Some(Ok(_)) = ready_sub.next() => {
                    self.update_klippy_status(KlippyState::Ready, &status_tx).await?;
                }
                Some(Ok(_)) = disconnected_sub.next() => {
                    self.update_klippy_status(KlippyState::Disconnected, &status_tx).await?;
                }
                Some(Ok(_)) = shutdown_sub.next() => {
                    self.update_klippy_status(KlippyState::Shutdown, &status_tx).await?;
                }
            }
        }
    }

    async fn update_klippy_status(
        &self,
        klippy_status: KlippyState,
        status_tx: &watch::Sender<Status>,
    ) -> Result<()> {
        match klippy_status {
            KlippyState::Ready => {
                self.register().await?;
                let status = self.client.get_printer_status().await?;
                status_tx.send_replace(Status::from(&status.status));
            }
            KlippyState::Disconnected => {
                status_tx.send_replace(Status {
                    printer: None,
                    state: State::Disconnected,
                });
            }
            _ => {}
        };
        Ok(())
    }

    async fn get_initial_klippy_state(&self) -> Result<KlippyState, anyhow::Error> {
        let info = self.client.get_server_info().await?;
        match info.klippy_state.as_str() {
            "ready" => Ok(KlippyState::Ready),
            "shutdown" => Ok(KlippyState::Shutdown),
            "disconnected" => Ok(KlippyState::Disconnected),
            _ => Err(anyhow::anyhow!(
                "unknown klippy state: {:?}",
                info.klippy_state
            )),
        }
    }

    async fn register(&self) -> Result<()> {
        self.client.register_printer_subscription().await?;
        self.client
            .register_remote_method(NOTIFICATION_METHOD)
            .await?;
        Ok(())
    }
}
