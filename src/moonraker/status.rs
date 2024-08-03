#[derive(Clone, Debug, Default)]
pub struct PrintInfo {
    pub current_layer: u16,
    pub total_layer: u16,
}

#[derive(Clone, Debug, Default)]
pub struct Printer {
    pub job: Option<PrintInfo>,
}

#[derive(Clone, Debug, Default, PartialEq)]
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
