#[derive(Clone, Debug, Default)]
pub struct Printer {
    pub current_layer: u16,
    pub total_layer: u16,
}

#[derive(Clone, Debug, Default)]
pub enum State {
    #[default]
    Disconnected,
    Startup,
    Standby,
    Printing,
    Paused,
    Complete,
    Shutdown(String),
    Error(String),
}

#[derive(Clone, Debug, Default)]
pub struct Status {
    pub printer: Option<Printer>,
    pub state: State,
}
