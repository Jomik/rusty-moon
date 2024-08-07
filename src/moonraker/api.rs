use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DisplayStatus {
    pub progress: Option<f64>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IdleTimeout {
    pub state: Option<String>,
}

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub struct PrintStatsInfo {
    pub current_layer: Option<u16>,
    pub total_layer: Option<u16>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PrintStats {
    pub info: Option<PrintStatsInfo>,
    pub state: Option<String>,
    pub filename: Option<String>,
    pub total_duration: Option<f64>,
    pub filament_used: Option<f64>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PrinterObjectStatus {
    pub display_status: Option<DisplayStatus>,
    pub idle_timeout: Option<IdleTimeout>,
    pub print_stats: PrintStats,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PrinterObjectStatusResponse {
    pub status: PrinterObjectStatus,
    #[serde(rename = "eventtime")]
    pub event_time: f64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IdentifyResult {
    pub connection_id: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ServerInfoResponse {
    pub klippy_connected: bool,
    pub klippy_state: String,
}
