use rusty_moon::connect_moonraker;

use tracing::Level;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish()
        .try_init()?;
    connect_moonraker().await?;

    // Start moonraker integration in its own thread
    // let moonraker = Moonraker::new("http://localhost:7125/").await?;

    // Start Discord integration in its own thread
    // let discord = Discord::new(token).await?;

    // moonraker.map(|event| bussiness_logic(event)).pipe.subscribe(discord).await?;
    // discord.map(business_logic).pipe.subscribe(moonraker).await?;

    Ok(())
}
