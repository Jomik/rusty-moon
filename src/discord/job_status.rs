use serenity::all::{CreateEmbed, CreateMessage, EditMessage};
use std::collections::HashMap;

use crate::moonraker::{JobInfo, State};

pub struct JobStatusMessage {
    embed: CreateEmbed,
}

struct ObjectData {
    pub total: u8,
    pub excluded: u8,
}

impl From<(State, JobInfo)> for JobStatusMessage {
    fn from(tuple: (State, JobInfo)) -> Self {
        let (state, job) = tuple;
        let mut object_map = HashMap::new();
        for object in job.objects.iter() {
            let name = object
                .name
                .split_once(".")
                .map(|(name, _)| name)
                .unwrap_or(object.name.as_str());
            let data = object_map.entry(name).or_insert(ObjectData {
                total: 0,
                excluded: 0,
            });
            data.total += 1;
            if object.excluded {
                data.excluded += 1;
            }
        }

        let mut embed = CreateEmbed::new()
            .title("Job Status")
            .field("State", state.to_string(), true)
            .field(
                "Layers",
                format!("{} / {}", job.current_layer, job.total_layer),
                true,
            );

        if !job.objects.is_empty() {
            embed = embed.field(
                format!(
                    "Objects {} / {}",
                    job.objects.iter().filter(|o| !o.excluded).count(),
                    job.objects.len()
                ),
                object_map
                    .into_iter()
                    .map(|(name, data)| {
                        format!("{}: {} / {}", name, data.total - data.excluded, data.total)
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
                false,
            );
        }

        JobStatusMessage { embed }
    }
}

impl From<JobStatusMessage> for CreateMessage {
    fn from(value: JobStatusMessage) -> Self {
        CreateMessage::new().embed(value.embed)
    }
}

impl From<JobStatusMessage> for EditMessage {
    fn from(value: JobStatusMessage) -> Self {
        EditMessage::new().embed(value.embed)
    }
}
