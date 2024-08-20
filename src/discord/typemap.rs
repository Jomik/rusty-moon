use std::sync::Arc;

use serenity::{
    all::{ChannelId, UserId},
    prelude::TypeMapKey,
};
use tokio::sync::{mpsc, watch, Mutex};

use crate::moonraker::{Notification, Status};

pub struct StatusChannel;
impl TypeMapKey for StatusChannel {
    type Value = Arc<Mutex<watch::Receiver<Status>>>;
}

pub struct NotificationChannel;
impl TypeMapKey for NotificationChannel {
    type Value = Arc<Mutex<mpsc::Receiver<Notification>>>;
}

pub struct OwnerId;
impl TypeMapKey for OwnerId {
    type Value = Arc<UserId>;
}

pub struct PrintsChannel;
impl TypeMapKey for PrintsChannel {
    type Value = Arc<ChannelId>;
}
