use std::fs::File;
use std::future::{Future, IntoFuture};
use std::io::Write;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use tokio::select;
use tokio::sync::{mpsc, watch};

use self::client::Client;
pub use self::status::*;

mod api;
mod client;
mod client_builder;
mod status;

const NOTIFICATION_METHOD: &str = "rusty_moon_notification";

#[derive(Debug, Clone, serde::Deserialize)]
struct NotificationParams {
    pub message: String,
    pub webcam: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub enum KlippyState {
    #[default]
    Disconnected,
    Ready,
    Shutdown,
}

#[derive(Debug, Default)]
pub struct Notification {
    pub message: String,
    pub image: Option<File>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub host: String,
    pub port: Option<u16>,
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
            let client = Arc::new(Client::builder(self.config).await?);

            Ok(Service { client })
        })
    }
}

pub struct Service {
    client: Arc<Client>,
}

impl Service {
    pub fn builder(config: Config) -> ServiceBuilder {
        ServiceBuilder::new(config)
    }

    pub async fn start(
        self,
        status_tx: watch::Sender<Status>,
        notification_tx: mpsc::Sender<Notification>,
    ) -> Result<()> {
        self.client.identify().await?;

        self.client
            .register_remote_method(NOTIFICATION_METHOD)
            .await?;
        let mut notification_sub = self
            .client
            .subscribe_remote_method::<NotificationParams>(NOTIFICATION_METHOD)
            .await?;
        let mut status_sub = self.client.subscribe_printer_status().await?;
        let mut ready_sub = self.client.subscribe_klippy_ready().await?;
        let mut disconnected_sub = self.client.subscribe_klippy_disconnected().await?;
        let mut shutdown_sub = self.client.subscribe_klippy_shutdown().await?;

        self.update_klippy_status(self.get_initial_klippy_state().await?, &status_tx)
            .await?;
        loop {
            // TODO: handle errors
            select! {
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
                opt = notification_sub.next() => {
                    match opt {
                        None => {},
                        Some(notif) => {
                            match notif {
                                Ok(notif) => {
                                    tracing::info!("received notification: {:?}", notif);
                                    let image = match notif.webcam {
                                        Some(webcam) => {
                                            let webcam = self.client.get_webcam_information(&webcam).await;
                                            match webcam {
                                                Ok(webcam) => {
                                                    tracing::debug!("webcam snapshot url: {:?}", webcam.snapshot_url);
                                                    let response = reqwest::get(webcam.snapshot_url).await?;
                                                    let mut file = tempfile::Builder::new().prefix("rusty_moon_").suffix(".jpeg").tempfile()?;
                                                    let content = response.bytes().await?;
                                                    file.write_all(&content)?;
                                                    tracing::debug!("saved webcam snapshot to {:?}", file.path());
                                                    Some(file.reopen()?)
                                                },
                                                Err(err) => {
                                                    tracing::error!("error getting webcam information: {}", err);
                                                    None
                                                }
                                            }
                                        },
                                        None => None,
                                    };

                                    notification_tx.send(Notification { message: notif.message, image }).await?;
                                }
                                Err(err) => {
                                    tracing::error!("error reading notification: {:?}", err);
                                }
                            }
                        }
                    }
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
            KlippyState::Shutdown => {
                // status_tx.send_replace(Status {
                //     printer: None,
                //     state: State::Shutdown(),
                // });
            }
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
        Ok(())
    }
}
