use std::str::FromStr;

use crate::external::*;
use crate::types::ISendParams;
use ethabi::{decode, ParamType, Token};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::Base64VecU8;
use near_sdk::{serde_json, PromiseOrValue};
use near_sdk::{env, json_types::U128, near_bindgen, AccountId, Gas, Promise};

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TestDapp {
    gateway: AccountId,
    route_token: AccountId,
    owner: AccountId,
    greeting_record: UnorderedMap<(String, u64), String>,
    ack_record: UnorderedMap<U128, bool>,
}

impl Default for TestDapp {
    // The default trait with which to initialize the contract
    fn default() -> Self {
        Self {
            gateway: AccountId::from_str("gateway").unwrap(),
            route_token: AccountId::from_str("route").unwrap(),
            owner: env::predecessor_account_id(),
            greeting_record: UnorderedMap::new(b"g"),
            ack_record: UnorderedMap::new(b"a"),
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl TestDapp {
    #[init]
    pub fn new(gateway: AccountId, route_token: AccountId) -> Self {
        Self {
            gateway,
            route_token,
            owner: env::predecessor_account_id(),
            greeting_record: UnorderedMap::new(b"g"),
            ack_record: UnorderedMap::new(b"a"),
        }
    }

    pub fn get_gateway(&self) -> AccountId {
        self.gateway.clone()
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn get_greeting_record(&self, src_chain_id: String, request_id: u64) -> String {
        self.greeting_record
            .get(&(src_chain_id, request_id))
            .unwrap_or("".to_string())
    }

    pub fn get_ack_record(&self, request_id: U128) -> bool {
        self.ack_record.get(&(request_id)).unwrap_or(false)
    }

    pub fn set_dapp_metadata(&self, fee_payer_address: String) -> Promise {
        if env::predecessor_account_id() != self.owner.clone() {
            env::panic_str("only owner");
        }

        gateway_contract::ext(self.gateway.clone())
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(5 * TGAS))
            .set_dapp_metadata(fee_payer_address)
    }

    pub fn send_i_request(
        &self,
        payload: Vec<u8>,
        dest_contract_address: String,
        dest_chain_id: String,
        request_metadata: Vec<u8>,
        amount: U128,
        route_recipient: String,
    ) -> Promise {
        let dest_contract_addr_token: Token = Token::String(dest_contract_address);
        let payload_token: Token = Token::Bytes(payload.clone());

        let param_types: Vec<ParamType> = vec![ParamType::Uint(64), ParamType::String];
        let decoded = decode(&param_types, &payload);

        if decoded.is_err() {
            env::panic_str(&format!(
                "Error in decoding payload: {:?}",
                decoded.unwrap_err()
            ));
        }

        let decoded: Vec<Token> = decoded.unwrap();
        let greeting: String = decoded[1].clone().into_string().unwrap();

        let request_packet: Vec<u8> = ethabi::encode(&[dest_contract_addr_token, payload_token]);

        if amount > U128::from(0) {
            if route_recipient == "".to_string() {
                env::panic_str("Route recipient cannot be empty");
            }

            let request_params: ISendParams = ISendParams {
                version: U128::from(1),
                route_recipient,
                request_metadata: request_metadata.clone(),
                request_packet: request_packet.clone(),
                dest_chain_id,
            };
            let encoded: String =
                near_sdk::base64::encode(serde_json::to_string(&request_params).unwrap());
            let decoded = near_sdk::base64::decode(encoded);

            if decoded.is_err() {
                env::panic_str(&format!(
                    "unable to decode the message: {:?}",
                    decoded.unwrap_err()
                ));
            }

            let msg: Base64VecU8 = Base64VecU8::from(decoded.unwrap());

            let promise: Promise = route_token::ext(self.route_token.clone())
                .with_static_gas(Gas(5 * TGAS))
                .burn_and_call_gateway(false, amount, msg);

            if greeting == "".to_string() {
                env::panic_str("greeting cannot be empty");
            }

            return promise;
        }

        let promise: Promise = gateway_contract::ext(self.gateway.clone())
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(Gas(5 * TGAS))
            .i_send(
                U128::from(1),
                dest_chain_id,
                request_metadata,
                request_packet,
            );

        if greeting == "".to_string() {
            env::panic_str("greeting cannot be empty");
        }

        promise
    }

    pub fn i_receive(
        &mut self,
        request_sender: String,
        packet: Vec<u8>,
        src_chain_id: String,
    ) -> PromiseOrValue<Vec<u8>> {
        if self.gateway.clone() != env::predecessor_account_id() {
            env::panic_str("only gateway");
        }

        let res = ethabi::decode(&[ParamType::Uint(64), ParamType::String], &packet);

        if res.is_err() {
            let error = res.unwrap_err();
            let format_error_string: String = format!("Error in decoding packet: {:?}", error);
            env::panic_str(&format_error_string);
        }

        let res: Vec<Token> = res.unwrap();
        let nonce: u64 = res[0].clone().into_uint().unwrap().as_u64();
        let greeting: String = res[1].clone().into_string().unwrap();

        if greeting == "Fail Dest Req".to_string() {
            env::panic_str("String != Fail Dest Req");
        }

        self.greeting_record
            .insert(&(src_chain_id, nonce), &greeting);

        PromiseOrValue::Value(packet)
    }

    pub fn i_ack(&mut self, request_identifier: U128, exec_flag: bool, exec_data: Vec<u8>) {
        if self.gateway.clone() != env::predecessor_account_id() {
            env::panic_str("only gateway");
        }

        if exec_flag {
            let res = ethabi::decode(&[ParamType::Uint(64), ParamType::String], &exec_data);

            if res.is_err() {
                let error = res.unwrap_err();
                let format_error_string: String = format!("Error in decoding packet: {:?}", error);
                env::panic_str(&format_error_string);
            }

            let res: Vec<Token> = res.unwrap();
            let greeting: String = res[1].clone().into_string().unwrap();

            if greeting == "Fail Ack Req".to_string() {
                env::panic_str("String != Fail Ack Req");
            }
        }

        self.ack_record.insert(&request_identifier, &true);
    }
}
