use anyhow::Result;
use std::{fs::File, io::Write};

use super::client::Client;

pub async fn get_webcam_snapshot(client: &Client, webcam: impl AsRef<str>) -> Result<Option<File>> {
    let info = client.get_webcam_information(&webcam).await?;
    tracing::debug!("webcam snapshot url: {:?}", info.snapshot_url);
    let response = reqwest::get(info.snapshot_url).await?;
    let mut file = tempfile::Builder::new()
        .prefix("rusty_moon_")
        .suffix(".jpeg")
        .tempfile()?;
    let content = response.bytes().await?;
    file.write_all(&content)?;
    tracing::debug!("saved webcam snapshot to {:?}", file.path());
    Ok(Some(file.reopen()?))
}
