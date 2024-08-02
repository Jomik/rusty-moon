use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::watch;
use tokio::task::{yield_now, JoinSet};

pub use self::api::*;
pub use self::status::*;

mod api;
mod client;
mod client_builder;
mod status;

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

    pub async fn start(&self, status_tx: watch::Sender<Status>) -> Result<()> {
        let client = self.client.clone();

        client.identify().await?;
        let mut tasks = JoinSet::new();

        {
            let client = client.clone();
            tasks.spawn(async move {
                // TODO: Handle subscription errors
                if let Err(err) = client
                    .subscribe_remote_method("rusty_moon_notification")
                    .await
                {
                    tracing::error!("Subscription error: {:?}", err);
                }
            });
        }

        let (status_notif_tx, mut status_notif_rx) =
            watch::channel::<PrinterStatusNotification>(PrinterStatusNotification::default());
        {
            let client = client.clone();

            tasks.spawn(async move {
                // TODO: Handle subscription errors
                if let Err(err) = client.subscribe_printer_status(status_notif_tx).await {
                    tracing::error!("Subscription error: {:?}", err);
                }
            });
        }

        loop {
            let notif = status_notif_rx.borrow_and_update().clone();
            status_tx.send_modify(|current| {
                current.state = match notif.status.print_stats.state.as_deref() {
                    None => State::default(),
                    Some("standby") => State::Standby,
                    Some("printing") => State::Printing,
                    // TODO: Handle paused during timelapses
                    Some("paused") => State::Paused,
                    Some("complete") => State::Complete,
                    Some("error") => {
                        State::Error(notif.status.print_stats.message.unwrap_or_default())
                    }
                    _ => State::Error(format!(
                        "unknown state: {:?}",
                        notif.status.print_stats.state
                    )),
                };

                if let Some(info) = notif.status.print_stats.info {
                    let _ = current.printer.insert(Printer {
                        current_layer: info.current_layer.unwrap_or_default(),
                        total_layer: info.total_layer.unwrap_or_default(),
                    });
                } else {
                    current.printer = None;
                }
            });

            yield_now().await;
            if status_notif_rx.changed().await.is_err() {
                break;
            }
        }

        tasks.shutdown().await;
        Ok(())
    }
}
