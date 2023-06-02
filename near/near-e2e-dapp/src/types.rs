use near_sdk::{
    json_types::U128,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
pub struct ISendParams {
    pub version: U128,
    pub route_recipient: String,
    pub dest_chain_id: String,
    pub request_metadata: Vec<u8>,
    pub request_packet: Vec<u8>,
}
