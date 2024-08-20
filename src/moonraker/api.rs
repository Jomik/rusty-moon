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
    #[serde(default)]
    pub info: PrintStatsInfo,
    pub state: Option<String>,
    #[serde(rename = "filename")]
    pub file_name: Option<String>,
    pub total_duration: Option<f64>,
    pub filament_used: Option<f64>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PrinterObjectStatus {
    #[serde(default)]
    pub display_status: DisplayStatus,
    #[serde(default)]
    pub idle_timeout: IdleTimeout,
    #[serde(default)]
    pub print_stats: PrintStats,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PrinterObjectStatusResponse {
    #[serde(default)]
    pub status: PrinterObjectStatus,
    #[serde(rename = "eventtime")]
    pub _event_time: f64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct IdentifyResult {
    #[serde(rename = "connection_id")]
    pub _connection_id: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ServerInfoResponse {
    #[serde(rename = "klippy_connected")]
    pub _klippy_connected: bool,
    pub klippy_state: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WebCamInformationResult {
    pub webcam: WebCamInformation,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WebCamInformation {
    pub snapshot_url: String,
}
