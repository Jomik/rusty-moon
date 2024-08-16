use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;
use serenity::all::{
    CreateAttachment, CreateEmbed, CreateMessage, EditMessage, GetMessages, Http, UserId,
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
        let user = UserId::new(self.config.user_id);
        Box::pin(async move {
            // let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
            // let client = Client::builder(self.config.token, intents).await?;
            let client = Arc::new(Http::new(&self.config.token));
            Ok(Service {
                client,
                user_id: user,
            })
        })
    }
}

pub struct Service {
    client: Arc<Http>,
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
        let user = self.client.get_user(self.user_id).await?;
        let bot = self.client.get_current_user().await?;
        let channel = user.create_dm_channel(&self.client).await?;
        let message_ids = channel
            .messages(&self.client, GetMessages::new())
            .await?
            .into_iter()
            .filter(|msg| msg.author.id == bot.id);

        for message in message_ids {
            message.delete(&self.client).await?;
        }

        let status = status_rx.borrow_and_update().clone();
        let message_builder = CreateMessage::new().embeds(self.get_status_embeds(status));
        let mut message = user.direct_message(&self.client, message_builder).await?;
        message.pin(&self.client).await?;

        loop {
            // TODO: handle errors
            select! {
                Ok(()) = status_rx.changed() =>{
                    let edit_builder = EditMessage::new().embeds(self.get_status_embeds(status_rx.borrow_and_update().clone()));
                    message.edit(&self.client, edit_builder).await?;
                },
                Some(notification) = notification_rx.recv() => {
                    let mut message_builder = CreateMessage::new().content(notification.message);
                    if let Some(image) =  notification.image {
                        message_builder = message_builder.add_file(CreateAttachment::file(&image.into(), "image.png").await?);
                    };
                    user.direct_message(&self.client, message_builder).await?;
                },
            }
        }
    }

    fn get_status_embeds(&self, status: moonraker::Status) -> Vec<CreateEmbed> {
        let printer_status = CreateEmbed::new().title("Printer status").field(
            "State",
            format!("{:?}", status.state),
            false,
        );

        let job_status = status.printer.map(|printer| {
            printer.job.map_or_else(
                || CreateEmbed::new().title("Job status").description("No job"),
                |job| {
                    CreateEmbed::new().title("Job status").field(
                        "Layer",
                        format!("{} / {}", job.current_layer, job.total_layer),
                        true,
                    )
                },
            )
        });

        vec![Some(printer_status), job_status]
            .into_iter()
            .flatten()
            .collect()
    }
}
