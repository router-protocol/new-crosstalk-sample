use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
    serde_json,
};
use std::fmt;

use crate::{CONTRACT_NAME, CONTRACT_VERSION};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
    PingFromSource(Vec<PingFromSourceEvent>),
    NewPing(Vec<NewPingEvent>),
    ExecutionStatus(Vec<ExecutionStatusEvent>),
    AckFromDestination(Vec<AckFromDestinationEvent>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub standard: String,
    pub version: String,

    // `flatten` to not have "event": {<EventLogVariant>} in the JSON, just have the contents of {<EventLogVariant>}.
    #[serde(flatten)]
    pub event: EventLogVariant,
}

impl EventLog {
    pub fn new(event: EventLogVariant) -> Self {
        Self {
            standard: CONTRACT_NAME.to_string(),
            version: CONTRACT_VERSION.to_string(),
            event,
        }
    }
}

impl fmt::Display for EventLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON:{}",
            &serde_json::to_string(self).map_err(|_| fmt::Error)?
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PingFromSourceEvent {
    pub src_chain_id: String,
    pub request_id: u64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NewPingEvent {
    pub request_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ExecutionStatusEvent {
    pub request_identifier: U128,
    pub is_success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct AckFromDestinationEvent {
    pub request_id: u64,
    pub ack_message: String,
}
