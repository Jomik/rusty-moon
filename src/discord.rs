mod typemap;

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
        ActivityData, Channel, ChannelId, ChannelType, Context, CreateAttachment, CreateEmbed,
        CreateMessage, CreateThread, EditMessage, EventHandler, GatewayIntents, GetMessages,
        GuildChannel, Message, OnlineStatus, Ready, UserId,
    },
    async_trait, Client,
};
use tokio::{
    select,
    sync::{mpsc, watch, Mutex},
};
use typemap::*;

use crate::moonraker::{self, State};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub token: String,
    pub user_id: u64,
    pub channel_id: u64,
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
            let user_id = UserId::new(self.config.user_id);
            let channel_id = ChannelId::new(self.config.channel_id);

            let client = Client::builder(self.config.token, GatewayIntents::default())
                .event_handler(Handler {
                    is_loop_running: AtomicBool::new(false),
                })
                .await?;
            Ok(Service {
                client,
                user_id,
                channel_id,
            })
        })
    }
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
    channel_id: ChannelId,
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
            data.insert::<PrintsChannel>(Arc::new(self.channel_id));

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
    let channel_id = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<PrintsChannel>()
            .unwrap()
            .as_ref()
            .to_owned()
    };
    let user = ctx.http.get_user(user_id).await?;
    let bot = ctx.http.get_current_user().await?;
    let channel = ctx.http.get_channel(channel_id).await?;

    {
        let dm_channel = user.create_dm_channel(ctx).await?;
        let message_ids = dm_channel
            .messages(ctx, GetMessages::new())
            .await?
            .into_iter()
            .filter(|msg| msg.author.id == bot.id);

        for message in message_ids {
            message.delete(ctx).await?;
        }
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
    set_presence(ctx, &status.state);
    let message_builder = CreateMessage::new().embeds(get_status_embeds(&status));
    let mut message = user.direct_message(ctx, message_builder).await?;
    message.pin(ctx).await?;

    let mut current_file_name = String::default();
    let mut thread: GuildChannel;
    let mut thread_status = Message::default();

    loop {
        let mut status_rx_lock = status_rx.lock().await;
        let mut notification_rx_lock = notification_rx.lock().await;
        // TODO: handle errors
        select! {
            Ok(()) = status_rx_lock.changed() => {
                let status = status_rx_lock.borrow_and_update().clone();
                set_presence(ctx, &status.state);

                let edit_builder = EditMessage::new().embeds(get_status_embeds(&status));
                message.edit(ctx, edit_builder.clone()).await?;

                if let Channel::Guild(channel) = channel.clone() {
                    if let Some(mut job) = status.clone().printer.and_then(|printer| printer.job) {
                        job = job.clone();
                        if job.file_name != current_file_name{
                            current_file_name = job.file_name;
                            thread = channel.create_thread(ctx, CreateThread::new(current_file_name.clone()).kind(ChannelType::PublicThread)).await?;
                            thread_status = thread.send_message(ctx, CreateMessage::new().embeds(get_status_embeds(&status))).await?;
                        } else {
                            thread_status.edit(ctx, edit_builder).await?;
                        }
                    }
                }
            },
            Some(notification) = notification_rx_lock.recv() => {
                let mut message_builder = CreateMessage::new().content(notification.message);
                if let Some(image) =  notification.image {
                    message_builder = message_builder.add_file(CreateAttachment::file(&image.into(), "image.png").await?);
                };
                user.direct_message(ctx, message_builder).await?;
            },
        }
    }
}

fn set_presence(ctx: &Context, state: &State) {
    match state {
        State::Disconnected => {
            ctx.set_presence(
                Some(ActivityData::custom("Disconnected")),
                OnlineStatus::Idle,
            );
        }
        State::Printing => {
            ctx.set_presence(
                Some(ActivityData::custom("Printing")),
                OnlineStatus::DoNotDisturb,
            );
        }
        State::Paused => {
            ctx.set_presence(Some(ActivityData::custom("Paused")), OnlineStatus::Online);
        }
        State::Startup | State::Standby | State::Complete => {
            ctx.set_presence(Some(ActivityData::custom("Ready")), OnlineStatus::Online);
        }
        State::Shutdown(_) => {
            ctx.set_presence(Some(ActivityData::custom("Shutdown")), OnlineStatus::Idle);
        }
        State::Error(_) => {
            ctx.set_presence(Some(ActivityData::custom("Error")), OnlineStatus::Idle);
        }
    };
}

fn get_status_embeds(status: &moonraker::Status) -> Vec<CreateEmbed> {
    let job_status = status.clone().printer.map(|printer| {
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

    vec![job_status].into_iter().flatten().collect()
}
