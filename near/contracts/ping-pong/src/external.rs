use near_sdk::{ext_contract, json_types::U128};

pub const TGAS: u64 = 1_000_000_000_000;

// Validator interface, for cross-contract calls
#[ext_contract(gateway_contract)]
trait GatewayContract {
    fn i_send(
        &mut self,
        version: U128,
        dest_chain_id: String,
        request_metadata: Vec<u8>,
        request_packet: Vec<u8>,
    ) -> bool;

    fn set_dapp_metadata(&self, fee_payer_address: String);
}
