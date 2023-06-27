mod events;
mod external;
#[cfg(test)]
mod tests;

use events::{
    AckFromDestinationEvent, EventLog,
    EventLogVariant::{AckFromDestination, ExecutionStatus, NewPing, PingFromSource},
    ExecutionStatusEvent, NewPingEvent, PingFromSourceEvent,
};
use external::*;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::UnorderedMap,
    env,
    json_types::U128,
    near_bindgen, AccountId, Gas, Promise,
};
use router_wasm_bindings::ethabi::{decode, encode, ethereum_types::U256, ParamType, Token};

pub const CONTRACT_VERSION: &str = "1.0.0";
pub const CONTRACT_NAME: &str = "PingPong";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct PingPong {
    owner: AccountId,
    gateway: AccountId,
    current_request_id: u64,
    // srcChainType, srcChainId, requestId -> pingFromSource
    ping_from_source: UnorderedMap<(String, u64), String>,
    // requestId => ackMessage
    ack_from_destination: UnorderedMap<u64, String>,
}

impl Default for PingPong {
    fn default() -> Self {
        Self {
            owner: env::predecessor_account_id(),
            gateway: env::predecessor_account_id(),
            current_request_id: 0,
            ping_from_source: UnorderedMap::new(b'p'),
            ack_from_destination: UnorderedMap::new(b'a'),
        }
    }
}

#[near_bindgen]
impl PingPong {
    #[init]
    pub fn new(gateway: AccountId) -> Self {
        Self {
            owner: env::predecessor_account_id(),
            gateway,
            current_request_id: 0,
            ping_from_source: UnorderedMap::new(b'p'),
            ack_from_destination: UnorderedMap::new(b'a'),
        }
    }

    pub fn set_gateway(&mut self, gateway: AccountId) {
        if env::predecessor_account_id() != self.owner.clone() {
            env::panic_str("only owner");
        }

        self.gateway = gateway;
    }

    pub fn get_current_request_id(&self) -> u64 {
        return self.current_request_id.clone();
    }

    pub fn get_ping_from_source(&self, src_chain_id: String, request_id: u64) -> String {
        self.ping_from_source
            .get(&(src_chain_id, request_id))
            .unwrap_or("".to_string())
    }

    pub fn get_ack_from_destination(&self, request_id: u64) -> String {
        self.ack_from_destination
            .get(&request_id)
            .unwrap_or("".to_string())
    }

    pub fn get_gateway(&self) -> AccountId {
        self.gateway.clone()
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    #[payable]
    pub fn set_dapp_metadata(&mut self, fee_payer_address: String) -> Promise {
        if env::predecessor_account_id() != self.owner.clone() {
            env::panic_str("only owner");
        }

        gateway_contract::ext(self.gateway.clone())
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(5 * TGAS))
            .set_dapp_metadata(fee_payer_address)
    }

    pub fn get_request_metadata(
        dest_gas_limit: u64,
        dest_gas_price: u64,
        ack_gas_limit: u64,
        ack_gas_price: u64,
        relayer_fees: U128,
        ack_type: u8,
        is_read_call: bool,
        asm_address: String,
    ) -> Vec<u8> {
        let mut request_metadata: Vec<u8> = vec![];

        request_metadata.append(&mut dest_gas_limit.to_be_bytes().to_vec());
        request_metadata.append(&mut dest_gas_price.to_be_bytes().to_vec());
        request_metadata.append(&mut ack_gas_limit.to_be_bytes().to_vec());
        request_metadata.append(&mut ack_gas_price.to_be_bytes().to_vec());
        request_metadata.append(&mut u128::from(relayer_fees).to_be_bytes().to_vec());
        request_metadata.append(&mut ack_type.to_be_bytes().to_vec());

        if is_read_call {
            request_metadata.append(&mut vec![1]);
        } else {
            request_metadata.append(&mut vec![0]);
        }
        request_metadata.append(&mut asm_address.as_bytes().to_vec());

        request_metadata
    }

    #[payable]
    pub fn i_ping(
        &mut self,
        dest_chain_id: String,
        destination_contract_address: String,
        str: String,
        request_metadata: Vec<u8>,
        recipient: String
    ) -> Promise {
        self.current_request_id += 1;

        let request_id_token: Token = Token::Uint(U256::from(self.current_request_id.clone()));
        let message_token: Token = Token::String(str);
        let recipient_token: Token = Token::String(recipient);

        // abi.encode(request_id, message)
        let packet: Vec<u8> = encode(&[request_id_token, message_token, recipient_token]);

        let handler_token: Token = Token::String(destination_contract_address);
        let packet_token: Token = Token::Bytes(packet);

        // abi.encode(packet, message)
        let request_packet: Vec<u8> = encode(&[handler_token, packet_token]);

        let ping_event: EventLog = EventLog::new(NewPing(vec![NewPingEvent {
            request_id: self.current_request_id.clone(),
        }]));

        env::log_str(&ping_event.to_string());

        gateway_contract::ext(self.gateway.clone())
            .with_attached_deposit(env::attached_deposit())
            .i_send(
                U128::from(1),
                dest_chain_id,
                request_metadata,
                request_packet,
            )
    }

    pub fn i_receive(
        &mut self,
        request_sender: String,
        packet: Vec<u8>,
        src_chain_id: String,
    ) -> Vec<u8> {
        if env::predecessor_account_id() != self.gateway.clone() {
            env::panic_str("not gateway");
        }

        let param_vec: Vec<ParamType> = vec![ParamType::Uint(64), ParamType::String];

        let token_vec: Vec<Token> = match decode(&param_vec, &packet) {
            Ok(data) => data,
            Err(_) => env::panic_str("not able to decode the packet"),
        };

        let request_id: u64 = token_vec[0].clone().into_uint().unwrap().as_u64();
        let message: String = token_vec[1].clone().into_string().unwrap();

        if message == "".to_string() {
            env::panic_str("String should not be empty");
        }

        self.ping_from_source
            .insert(&(src_chain_id.clone(), request_id.clone()), &message);

        let ping_from_source: EventLog = EventLog::new(PingFromSource(vec![PingFromSourceEvent {
            src_chain_id: src_chain_id.clone(),
            request_id: request_id,
            message: message,
        }]));

        env::log_str(&ping_from_source.to_string());

        packet
    }

    pub fn i_ack(
        &mut self,
        request_identifier: U128,
        exec_flag: bool,
        exec_data: Vec<u8>,
    ) {
        if env::predecessor_account_id() != self.gateway.clone() {
            env::panic_str("not gateway");
        }

        let decoded = decode(&[ParamType::Uint(64), ParamType::String], &exec_data);
        if decoded.is_err() {
            let format_str: String = format!(
                "Cannot decode the exec data for request_id: {:?}",
                request_identifier.clone()
            );
            env::panic_str(&format_str);
        }

        let decoded: Vec<Token> = decoded.unwrap();
        let request_id: u64 = decoded[0].clone().into_uint().unwrap().as_u64();
        let ack_message: String = decoded[1].clone().into_string().unwrap();

        self.ack_from_destination.insert(&request_id, &ack_message);

        let exec_status_event: EventLog =
            EventLog::new(ExecutionStatus(vec![ExecutionStatusEvent {
                request_identifier: request_identifier.clone(),
                is_success: exec_flag,
            }]));

        env::log_str(&exec_status_event.to_string());

        let ack_from_destination_event: EventLog =
            EventLog::new(AckFromDestination(vec![AckFromDestinationEvent {
                request_id,
                ack_message,
            }]));

        env::log_str(&ack_from_destination_event.to_string());
    }

    pub fn withdraw_fees(&self, recipient: AccountId) -> Promise {
        if env::predecessor_account_id() != self.owner {
            env::panic_str("Only owner");
        }

        let balance: u128 = env::account_balance() - Self::total_storage_cost();
        Promise::new(recipient).transfer(balance)
    }

    pub fn total_storage_cost() -> u128 {
        u128::from(env::storage_usage()) * env::storage_byte_cost()
    }
}
