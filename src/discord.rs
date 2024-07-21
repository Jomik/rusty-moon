use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

use anyhow::Result;
use serenity::{
    all::{CreateMessage, GatewayIntents, UserId},
    Client,
};
use tokio::{sync::mpsc, task::yield_now};

use crate::moonraker::Event;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub token: String,
    pub user_id: u64,
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
        let intents = GatewayIntents::DIRECT_MESSAGES;
        let user = UserId::new(self.config.user_id);
        Box::pin(async move {
            // let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
            let client = Client::builder(self.config.token, intents).await?;
            Ok(Service { client, user })
        })
    }
}

pub struct Service {
    client: Client,
    user: UserId,
}

impl Service {
    pub fn builder(config: Config) -> ServiceBuilder {
        ServiceBuilder::new(config)
    }

    pub async fn start(self, mut events_rx: mpsc::Receiver<Event>) -> Result<()> {
        let http = self.client.http;
        while let Some(event) = events_rx.recv().await {
            let content = match event {
                Event::LayerChanged(layer) => {
                    if layer.is_none() {
                        continue;
                    }
                    Some(format!("Layer: {:?}", layer.unwrap()))
                }
                Event::PrinterStatusChanged(_status) => {
                    // Ignore this
                    None
                }
            };
            if let Some(content) = content {
                tracing::info!("Sending message: {:?}", content);
                let builder = CreateMessage::new().content(content);
                self.user.direct_message(http.clone(), builder).await?;
            }
            yield_now().await;
        }

        Ok(())
    }
}
