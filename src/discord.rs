use std::{
    future::{Future, IntoFuture},
    pin::Pin,
};

use anyhow::Result;
use serenity::{
    all::{CreateMessage, EditMessage, GatewayIntents, UserId},
    Client,
};
use tokio::{
    select,
    sync::{mpsc, watch},
};

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

    pub async fn start(
        self,
        mut status_rx: watch::Receiver<moonraker::Status>,
        mut notification_rx: mpsc::Receiver<moonraker::Notification>,
    ) -> Result<()> {
        let http = self.client.http.clone();
        let user = self.user_id.to_user(http.clone()).await?;

        let status = status_rx.borrow_and_update().clone();
        let message_builder = CreateMessage::new().content(self.get_status_message(status));
        let mut message = user.direct_message(http.clone(), message_builder).await?;

        loop {
            // TODO: handle errors
            select! {
                Ok(()) = status_rx.changed() =>{
                    let edit_builder = EditMessage::new().content(self.get_status_message(status_rx.borrow().clone()));
                    message.edit(http.clone(), edit_builder).await?;
                },
                Some(notification) = notification_rx.recv() => {
                    let message_builder = CreateMessage::new().content(notification.message);
                    message = user.direct_message(http.clone(), message_builder).await?;
                },
            }
        }
    }

    fn get_status_message(&self, status: moonraker::Status) -> String {
        format!("{:?}", status)
    }
}
