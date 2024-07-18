pub mod config;
pub mod moonraker;

// pub async fn connect_moonraker() -> anyhow::Result<()> {
//     let conf = config::load()?;
//
//     let mut status_watcher = moonraker::connect(conf.moonraker.into_owned()).await?;
//
//     let mut current_status = moonraker::PrintStatsInfo::default();
//     loop {
//         let next = status_watcher.borrow_and_update().status.print_stats.info;
//         if next.current_layer != current_status.current_layer {
//             tracing::info!("Layer: {:?}", next.current_layer);
//             current_status = next;
//         }
//         if status_watcher.changed().await.is_err() {
//             break;
//         }
//     }
//     Ok(())
// }
