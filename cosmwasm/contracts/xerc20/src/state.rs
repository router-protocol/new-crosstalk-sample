use cw_storage_plus::{Item, Map};

pub const INSTANTIATE_REPLY_ID: u64 = 1;
pub const CREATE_I_SEND_REQUEST: u64 = 2;

pub const OWNER: Item<String> = Item::new("owner");

pub const WHITELISTED_CONTRACT_MAPPING: Map<&str, String> = Map::new("forwarder_contract_mapping");

pub const CHAIN_TYPE_MAPPING: Map<&str, u64> = Map::new("chain_type_mapping");

pub const CHAIN_ID: Item<String> = Item::new("chain_id");

pub const CROSS_CHAIN_TOKEN: Item<String> = Item::new("cross_chain_token");
