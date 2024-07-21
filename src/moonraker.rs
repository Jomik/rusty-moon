use std::future::{Future, IntoFuture};
use std::pin::Pin;

use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::yield_now;

pub use self::api::*;

mod api;
mod client;
mod client_builder;

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
            let client = client::Client::builder(self.config.url).await?;

            Ok(Service { client })
        })
    }
}

pub struct Service {
    client: client::Client,
}

#[derive(Debug)]
pub enum Event {
    LayerChanged(Option<u16>),
    PrinterStatusChanged(PrinterObjectStatus),
}

impl Service {
    pub fn builder(config: Config) -> ServiceBuilder {
        ServiceBuilder::new(config)
    }

    pub async fn start(self, events_tx: mpsc::Sender<Event>) -> Result<()> {
        let client = self.client;

        let (status_tx, mut status_rx) = mpsc::channel::<PrinterStatusNotification>(100);

        tokio::spawn(async move {
            // TODO: Handle subscription errors
            if let Err(err) = client.subscribe_printer_status(status_tx).await {
                tracing::error!("Subscription error: {:?}", err);
            }
        });

        let current_status = PrinterObjectStatus::default();
        while let Some(printer) = status_rx.recv().await {
            events_tx
                .send(Event::PrinterStatusChanged(printer.status.clone()))
                .await?;
            if let Some(info) = printer.status.print_stats.info {
                if info.current_layer
                    != current_status
                        .print_stats
                        .info
                        .unwrap_or(PrintStatsInfo {
                            current_layer: None,
                            total_layer: None,
                        })
                        .current_layer
                {
                    events_tx
                        .send(Event::LayerChanged(info.current_layer))
                        .await?;
                }
            }

            yield_now().await;
        }
        Ok(())
    }
}
