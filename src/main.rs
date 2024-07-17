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
    Ok(())
}
