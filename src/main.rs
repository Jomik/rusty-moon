use tokio::sync::{mpsc, watch};
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

use rusty_moon::{config, discord, moonraker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf = config::load()?;

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .pretty()
        .finish()
        .try_init()?;

    let (status_tx, status_rx) = watch::channel(moonraker::Status::default());
    let (notification_tx, notification_rx) = mpsc::channel(10);

    let moon = moonraker::Service::builder(conf.moonraker).await?;
    tokio::spawn(async move {
        if let Err(err) = moon.start(status_tx, notification_tx).await {
            tracing::error!("Moonraker error: {:?}", err);
        }
    });

    let discord = discord::Service::builder(conf.discord).await?;
    if let Err(err) = discord.start(status_rx, notification_rx).await {
        tracing::error!("Discord error: {:?}", err);
    }

    Ok(())
}
