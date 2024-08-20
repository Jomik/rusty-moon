use std::{
    future::{Future, IntoFuture},
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::Result;
use serenity::{
    all::{
        ActivityData, Context, CreateAttachment, CreateEmbed, CreateMessage, EditMessage,
        EventHandler, GatewayIntents, GetMessages, OnlineStatus, Ready, UserId,
    },
    async_trait,
    prelude::TypeMapKey,
    Client,
};
use tokio::{
    select,
    sync::{mpsc, watch, Mutex},
};

use crate::moonraker::{self, State};

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
        Box::pin(async move {
            let user = UserId::new(self.config.user_id);

            let client = Client::builder(self.config.token, GatewayIntents::default())
                .event_handler(Handler {
                    is_loop_running: AtomicBool::new(false),
                })
                .await?;
            Ok(Service {
                client,
                user_id: user,
            })
        })
    }
}

struct StatusChannel;
impl TypeMapKey for StatusChannel {
    type Value = Arc<Mutex<watch::Receiver<moonraker::Status>>>;
}

struct NotificationChannel;
impl TypeMapKey for NotificationChannel {
    type Value = Arc<Mutex<mpsc::Receiver<moonraker::Notification>>>;
}

struct OwnerId;
impl TypeMapKey for OwnerId {
    type Value = Arc<UserId>;
}

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data: Ready) {
        let ctx = Arc::new(ctx);
        if self.is_loop_running.load(Ordering::Relaxed) {
            return;
        }

        let ctx = Arc::clone(&ctx);
        tokio::spawn(async move { run(&ctx).await });

        self.is_loop_running.swap(true, Ordering::Relaxed);
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
        mut self,
        status_rx: watch::Receiver<moonraker::Status>,
        notification_rx: mpsc::Receiver<moonraker::Notification>,
    ) -> Result<()> {
        {
            let mut data = self.client.data.write().await;

            data.insert::<OwnerId>(Arc::new(self.user_id));
            data.insert::<StatusChannel>(Arc::new(Mutex::new(status_rx)));
            data.insert::<NotificationChannel>(Arc::new(Mutex::new(notification_rx)));
        }
        self.client.start().await?;
        Ok(())
    }
}

async fn run(ctx: &Context) -> Result<()> {
    let user_id = {
        let data_read = ctx.data.read().await;
        data_read.get::<OwnerId>().unwrap().as_ref().to_owned()
    };
    let user = ctx.http.get_user(user_id).await?;
    let bot = ctx.http.get_current_user().await?;

    let channel = user.create_dm_channel(&ctx.http).await?;
    let message_ids = channel
        .messages(&ctx.http, GetMessages::new())
        .await?
        .into_iter()
        .filter(|msg| msg.author.id == bot.id);

    for message in message_ids {
        message.delete(&ctx.http).await?;
    }

    let status_rx = {
        let data_read = ctx.data.read().await;
        data_read.get::<StatusChannel>().unwrap().clone()
    };

    let notification_rx = {
        let data_read = ctx.data.read().await;
        data_read.get::<NotificationChannel>().unwrap().clone()
    };

    let status = status_rx.lock().await.borrow_and_update().clone();
    let message_builder = CreateMessage::new().embeds(get_status_embeds(status));
    let mut message = user.direct_message(&ctx.http, message_builder).await?;
    message.pin(&ctx.http).await?;

    loop {
        let mut status_rx_lock = status_rx.lock().await;
        let mut notification_rx_lock = notification_rx.lock().await;
        // TODO: handle errors
        select! {
            Ok(()) = status_rx_lock.changed() => {
                let status = status_rx_lock.borrow_and_update().clone();
                match status.state {
                    State::Disconnected | State::Shutdown(_) | State::Error(_) => {
                        ctx.set_presence(Some(ActivityData::custom("Disconnected")), OnlineStatus::Offline);
                    },
                    State::Printing => {
                        ctx.set_presence(Some(ActivityData::custom("Printing")), OnlineStatus::DoNotDisturb);
                    },
                    State::Paused => {
                        ctx.set_presence(Some(ActivityData::custom("Paused")), OnlineStatus::Idle);
                    },
                    State::Startup | State::Standby | State::Complete => {
                        ctx.set_presence(Some(ActivityData::custom("Standby")), OnlineStatus::Online);
                    },
                }

                let edit_builder = EditMessage::new().embeds(get_status_embeds(status));
                message.edit(&ctx.http, edit_builder).await?;
            },
            Some(notification) = notification_rx_lock.recv() => {
                let mut message_builder = CreateMessage::new().content(notification.message);
                if let Some(image) =  notification.image {
                    message_builder = message_builder.add_file(CreateAttachment::file(&image.into(), "image.png").await?);
                };
                user.direct_message(&ctx.http, message_builder).await?;
            },
        }
    }
}

fn get_status_embeds(status: moonraker::Status) -> Vec<CreateEmbed> {
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
