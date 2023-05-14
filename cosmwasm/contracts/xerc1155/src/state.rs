use cw_storage_plus::{Item, Map};

pub const OWNER: Item<String> = Item::new("owner");
pub const CW1155_CONTRACT_ADDRESS: Item<String> = Item::new("cw1155_contract_address");
// chain chain id => address of our contract in bytes
pub const REMOTE_CONTRACT_MAPPING: Map<String, String> = Map::new("remote_contract_mapping");
