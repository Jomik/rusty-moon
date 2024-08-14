use super::{PrintStats, PrinterObjectStatus};

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

impl From<&PrintStats> for State {
    fn from(value: &PrintStats) -> Self {
        match value.state.as_deref() {
            None => Self::default(),
            Some("standby") => Self::Standby,
            Some("printing") => Self::Printing,
            // TODO: Handle timelapse pauses
            Some("paused") => Self::Paused,
            Some("complete") => Self::Complete,
            Some("error") => Self::Error(value.message.clone().unwrap_or_default()),
            _ => Self::Error(format!("unknown state: {:?}", value.state)),
        }
    }
}

impl From<&PrintStats> for Printer {
    fn from(value: &PrintStats) -> Self {
        Self {
            job: match State::from(value) {
                State::Printing | State::Paused | State::Complete => Some(PrintInfo {
                    current_layer: value.info.current_layer.unwrap_or_default(),
                    total_layer: value.info.total_layer.unwrap_or_default(),
                }),
                _ => None,
            },
        }
    }
}

impl From<&PrinterObjectStatus> for Status {
    fn from(value: &PrinterObjectStatus) -> Self {
        Self {
            printer: Some(Printer::from(&value.print_stats)),
            state: State::from(&value.print_stats),
        }
    }
}
