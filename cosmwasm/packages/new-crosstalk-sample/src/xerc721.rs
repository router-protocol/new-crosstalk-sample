// use crate::{Deserialize, Serialize};
// use schemars::JsonSchema;

// use cosmwasm_std::Uint128;

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct TicketMetadata {
//     pub participant: String,
//     pub rider_id: Uint128,
//     pub score: Uint128,
//     pub lower_bound: u128,
//     pub upper_bound: u128,
// }

// // Define struct LotteryMetadata
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct LotteryMetadata {
//     pub start_time: u64,
//     pub end_time: u64,
//     pub claimed: bool,
//     pub lucky_number: u128,
//     pub winner: String,
//     pub reward: Uint128,
//     pub ticket_ranger: u128,
// }

// // Define storage key prefix
// const LOTTERY_METADATA_PREFIX: &[u8] = b"lottery_metadata";

// // Define message for adding multiple lotteries
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub struct AddMLotteryMsg {
//     pub rewards: Vec<Uint128>,
// }

// // Define state for last lottery time and lottery unique limit

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct InstantiateMsg {
//     pub owner: String,
//     pub _pd: u64,
//     pub _ld: u64,
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum ExecuteMsg {
//     AddLotteries {
//         count: u64,
//         rewards: Vec<Uint128>,
//     },
//     DrawLuckyNumber {},
//     ClaimLottery {
//         lottery_id: u64,
//         rider_id: Uint128,
//     },
//     SubmitTicket {
//         rider_id: Uint128,
//         lottery_id: u64,
//         score: Uint128,
//         sender: String,
//     },
//     EnrollRemoteContract {
//         chain_id: String,
//         remote_address: String,
//     },
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct MigrateMsg {}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum QueryMsg {
//     // fetch contract version
//     GetContractVersion {},
// }
