#![allow(dead_code)]
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub struct PrintStatsInfo {
    pub current_layer: Option<u64>,
    pub total_layer: Option<u64>,
}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub struct PrintStats {
    pub info: PrintStatsInfo,
}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub struct PrinterStatus {
    pub print_stats: PrintStats,
}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub struct PrinterStatusNotification {
    pub status: PrinterStatus,
    #[serde(rename = "eventtime")]
    pub event_time: u64,
}
