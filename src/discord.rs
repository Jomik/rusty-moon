use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

use anyhow::Result;
use serenity::{
    all::{CreateMessage, EditMessage, GatewayIntents, UserId},
    Client,
};
use tokio::sync::watch;

use crate::moonraker;

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
            Ok(Service {
                client,
                user_id: user,
            })
        })
    }
}

pub struct Service {
    client: Client,
    user_id: UserId,
}

impl Service {
    pub fn builder(config: Config) -> ServiceBuilder {
        ServiceBuilder::new(config)
    }

    pub async fn start(self, mut status_rx: watch::Receiver<moonraker::Status>) -> Result<()> {
        let http = self.client.http;
        let user = self.user_id.to_user(http.clone()).await?;

        let message_builder = CreateMessage::new().content("Hello!");
        let mut message = user.direct_message(http.clone(), message_builder).await?;

        loop {
            let status = status_rx.borrow_and_update().clone();
            let edit_builder = EditMessage::new().content(format!("{:?}", status));
            message.edit(http.clone(), edit_builder).await?;
            if status_rx.changed().await.is_err() {
                break;
            }
        }

        Ok(())
    }
}
