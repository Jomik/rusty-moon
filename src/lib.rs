mod config;
mod moonraker;

pub async fn connect_moonraker() -> anyhow::Result<()> {
    let conf = config::load()?;

    moonraker::connect(conf.moonraker.into_owned()).await?;

    Ok(())
}
