use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use jsonrpsee::core::client::Subscription;
use serde_json::Value;
use tokio::select;
use tokio::sync::watch;
use tokio::task::{yield_now, JoinSet};

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

        let (klippy_status_tx, mut klippy_status_rx) = watch::channel(KlippyState::default());
        let _notification_sub: Subscription<Value> =
            client.subscribe_remote_method(NOTIFICATION_METHOD).await?;
        let mut status_sub = client.subscribe_printer_status().await?;

        let mut klippy_tasks = self.watch_klippy(klippy_status_tx).await?;

        self.update_klippy_status(&mut klippy_status_rx, &status_tx)
            .await?;
        loop {
            select! {
                res = klippy_status_rx.changed() => {
                    if res.is_err() {
                        break;
                    }
                    self.update_klippy_status(&mut klippy_status_rx, &status_tx).await?;
                }
                Some(Ok(notif)) = status_sub.next() => {
                    self.update_printer_status(&status_tx, notif).await?;
                }
            }
            yield_now().await;
            if klippy_status_rx.changed().await.is_err() {
                break;
            }
        }

        klippy_tasks.shutdown().await;

        Ok(())
    }

    async fn update_klippy_status(
        &self,
        klippy_status_rx: &mut watch::Receiver<KlippyState>,
        status_tx: &watch::Sender<Status>,
    ) -> Result<()> {
        let klippy = klippy_status_rx.borrow_and_update().clone();
        match klippy {
            KlippyState::Ready => {
                self.register().await?;
                let status = self.client.get_printer_status().await?;
                self.update_printer_status(status_tx, status).await?;
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

    async fn update_printer_status(
        &self,
        status_tx: &watch::Sender<Status>,
        notif: PrinterObjectStatusResponse,
    ) -> Result<()> {
        status_tx.send_modify(|current| {
            current.state = match notif.status.print_stats.state.as_deref() {
                None => State::default(),
                Some("standby") => State::Standby,
                Some("printing") => State::Printing,
                // TODO: Handle timelapse pauses
                Some("paused") => State::Paused,
                Some("complete") => State::Complete,
                Some("error") => State::Error(notif.status.print_stats.message.unwrap_or_default()),
                _ => State::Error(format!(
                    "unknown state: {:?}",
                    notif.status.print_stats.state
                )),
            };

            if let Some(info) = notif.status.print_stats.info {
                let _ = current.printer.insert(Printer {
                    job: match current.state {
                        State::Printing | State::Paused | State::Complete => Some(PrintInfo {
                            current_layer: info.current_layer.unwrap_or_default(),
                            total_layer: info.total_layer.unwrap_or_default(),
                        }),
                        _ => None,
                    },
                });
            } else {
                current.printer = None;
            }
        });
        Ok(())
    }

    async fn watch_klippy(
        &self,
        status_tx: watch::Sender<KlippyState>,
    ) -> Result<JoinSet<Result<()>>> {
        let client = self.client.clone();
        let initial = self.client.get_server_info().await?;
        match initial.klippy_state.as_str() {
            "ready" => status_tx.send_replace(KlippyState::Ready),
            "shutdown" => status_tx.send_replace(KlippyState::Shutdown),
            _ => status_tx.send_replace(KlippyState::Disconnected),
        };
        let mut tasks = JoinSet::new();

        {
            let client = client.clone();
            let status_tx = status_tx.clone();
            tasks.spawn(async move {
                let mut sub = client.subscribe_klippy_ready().await?;
                while let Some(res) = sub.next().await {
                    res?;
                    status_tx.send_replace(KlippyState::Ready);
                }
                Ok(()) as Result<()>
            });
        }
        {
            let client = client.clone();
            let status_tx = status_tx.clone();
            tasks.spawn(async move {
                let mut sub = client.subscribe_klippy_disconnected().await?;
                while let Some(res) = sub.next().await {
                    res?;
                    status_tx.send_replace(KlippyState::Disconnected);
                }
                Ok(()) as Result<()>
            });
        }
        {
            let client = client.clone();
            let status_tx = status_tx.clone();
            tasks.spawn(async move {
                let mut sub = client.subscribe_klippy_shutdown().await?;
                while let Some(res) = sub.next().await {
                    res?;
                    status_tx.send_replace(KlippyState::Disconnected);
                }
                Ok(()) as Result<()>
            });
        }

        Ok(tasks)
    }

    async fn register(&self) -> Result<()> {
        self.client.register_printer_subscription().await?;
        self.client
            .register_remote_method(NOTIFICATION_METHOD)
            .await?;
        Ok(())
    }
}
