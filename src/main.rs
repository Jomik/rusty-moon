use tokio::sync::mpsc;
use tokio::task::yield_now;
use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

use rusty_moon::moonraker::Service;
use rusty_moon::{config, moonraker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf = config::load()?;

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .pretty()
        .finish()
        .try_init()?;

    let (events_tx, mut events_rx) = mpsc::channel::<moonraker::Event>(100);

    let moon = Service::builder(conf.moonraker).await?;
    tokio::spawn(async move {
        if let Err(err) = moon.start(events_tx).await {
            tracing::error!("Moonraker error: {:?}", err);
        }
    });

    while let Some(event) = events_rx.recv().await {
        match event {
            moonraker::Event::LayerChanged(layer) => {
                tracing::info!("Layer: {:?}", layer);
            }
            moonraker::Event::PrinterStatusChanged(status) => {
                tracing::info!("Status: {:?}", status);
            }
        }
        yield_now().await;
    }

    // Start moonraker integration in its own thread
    // let moonraker = Moonraker::new("http://localhost:7125/").await?;

    // Start Discord integration in its own thread
    // let discord = Discord::new(token).await?;

    // moonraker.map(|event| bussiness_logic(event)).pipe.subscribe(discord).await?;
    // discord.map(business_logic).pipe.subscribe(moonraker).await?;

    Ok(())
}
