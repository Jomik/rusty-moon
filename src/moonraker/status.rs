use core::fmt;
use std::fmt::{Display, Formatter};

use super::api::{ExcludeObject, PrintStats, PrinterObjectStatus};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjectInformation {
    pub name: String,
    pub excluded: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JobInfo {
    pub file_name: String,
    pub current_layer: u16,
    pub total_layer: u16,
    pub objects: Vec<ObjectInformation>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Printer {
    pub job: Option<JobInfo>,
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Status {
    pub printer: Option<Printer>,
    pub state: State,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Startup => write!(f, "Startup"),
            Self::Standby => write!(f, "Standby"),
            Self::Printing => write!(f, "Printing"),
            Self::Paused => write!(f, "Paused"),
            Self::Complete => write!(f, "Complete"),
            Self::Shutdown(reason) => write!(f, "Shutdown: {}", reason),
            Self::Error(message) => write!(f, "Error: {}", message),
        }
    }
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

impl From<PrinterObjectStatus> for Printer {
    fn from(value: PrinterObjectStatus) -> Self {
        Self {
            job: match State::from(&value.print_stats) {
                State::Printing | State::Paused | State::Complete => Some(JobInfo {
                    current_layer: value.print_stats.info.current_layer.unwrap_or_default(),
                    total_layer: value.print_stats.info.total_layer.unwrap_or_default(),
                    file_name: value
                        .print_stats
                        .file_name
                        .clone()
                        .unwrap_or("unknown".to_string()),
                    objects: (&value.exclude_object).into(),
                }),
                _ => None,
            },
        }
    }
}

impl From<&ExcludeObject> for Vec<ObjectInformation> {
    fn from(value: &ExcludeObject) -> Self {
        value
            .objects
            .iter()
            .map(|object| ObjectInformation {
                name: object.name.clone(),
                excluded: value.excluded_objects.contains(&object.name),
            })
            .collect()
    }
}

impl From<&PrinterObjectStatus> for Status {
    fn from(value: &PrinterObjectStatus) -> Self {
        Self {
            printer: Some(Printer::from(value.clone())),
            state: State::from(&value.print_stats),
        }
    }
}
