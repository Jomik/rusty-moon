use tokio::sync::mpsc;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

use rusty_moon::{config, discord, moonraker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf = config::load()?;

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .pretty()
        .finish()
        .try_init()?;

    let (events_tx, events_rx) = mpsc::channel::<moonraker::Event>(100);

    let moon = moonraker::Service::builder(conf.moonraker).await?;
    tokio::spawn(async move {
        if let Err(err) = moon.start(events_tx).await {
            tracing::error!("Moonraker error: {:?}", err);
        }
    });

    let discord = discord::Service::builder(conf.discord).await?;
    if let Err(err) = discord.start(events_rx).await {
        tracing::error!("Discord error: {:?}", err);
    }

    Ok(())
}
