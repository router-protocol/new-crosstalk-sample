use cw_storage_plus::{Item, Map};

pub const CREATE_I_SEND_REQUEST: u64 = 1;
// PingMapping (src_chain_id, requestId) => pingFromSource
pub const PING_FROM_SOURCE: Map<(&str, u64), String> = Map::new("ping_from_source");

// PongMapping requestId => pongFromSource
pub const PONG_FROM_DESTINATION: Map<&str, String> = Map::new("pong_from_destination");

pub const REQUEST_ID: Item<u64> = Item::new("request_id");
