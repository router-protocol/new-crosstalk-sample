use crate::{Deserialize, Serialize};
use schemars::JsonSchema;

use cosmwasm_std::{Binary, Uint128};

// Define state for last lottery time and lottery unique limit

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SendIRequest {
        payload: Binary,
        dest_contract_address: String,
        dest_chain_id: String,
        request_metadata: Binary,
        amount: Uint128,
        route_recipient: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // fetch contract version
    GetContractVersion {},
    FetchGreetingRecord { chain_id: String, request_id: u64 },
    FetchAckRecord { request_id: u64 },
}
