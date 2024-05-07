#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod cpi;
mod types;

#[ink::contract]
pub mod test_dapp {
    use crate::cpi::_call_i_send;
    use crate::cpi::_call_set_dapp_metadata;
    use crate::cpi::{_approve, _transfer_from};
    use crate::types::types::Result;
    use crate::types::TestError;
    use crate::types::{Bytes, _get_utf8_bytes_to_string};
    use ethabi::decode;
    use ethabi::encode;
    use ethabi::ParamType;
    use ethabi::Token;
    use ink::prelude::string::String;
    use ink::prelude::vec;
    use ink::prelude::vec::Vec as InkVec;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct TestDapp {
        pub route_token: AccountId,
        /// gateway contract address
        pub gateway_contract: AccountId,
        // request_identifer -> 0: sent, 1: success, 2: failed on dst
        pub greeting_record: Mapping<(String, u64), String>,
        pub ack_record_record: Mapping<u128, bool>,
    }

    impl TestDapp {
        #[ink(constructor)]
        pub fn new(
            gateway_contract: AccountId,
            fee_payer_address: Bytes,
            route_token: AccountId,
        ) -> Self {
            let mut this = Self {
                route_token,
                gateway_contract,
                ack_record_record: Mapping::default(),
                greeting_record: Mapping::default(),
            };
            this.set_dapp_metadata(fee_payer_address);
            this
        }

        #[ink(message, payable)]
        pub fn set_dapp_metadata(&mut self, fee_payer_address: Bytes) {
            _call_set_dapp_metadata(
                self.gateway_contract,
                self.env().transferred_value(),
                fee_payer_address,
            )
        }

        #[ink(message, payable)]
        pub fn i_send(
            &mut self,
            route_amount: u128,
            route_recipient: Bytes,
            dest_chain_id: Bytes,
            request_metadata: Bytes,
            payload: Bytes,
            dst_contract: Bytes,
        ) -> Result<u128> {
            let request_packet = encode(&[
                Token::String(_get_utf8_bytes_to_string(&dst_contract)),
                Token::Bytes(payload.clone()),
            ]);
            let decoded_data = decode(&[ParamType::Uint(64), ParamType::String], &payload);
            if decoded_data.is_err() {
                return Err(TestError::DecodeError);
            }
            let decoded_data = decoded_data.unwrap();
            if decoded_data[1].clone().into_string().unwrap() == String::from("") {
                return Err(TestError::EmptyGreeting);
            }
            if route_amount > 0 {
                _transfer_from(
                    self.route_token,
                    self.env().caller(),
                    self.env().account_id(),
                    route_amount,
                    &[],
                )?;
                _approve(self.route_token, self.gateway_contract, route_amount)?;
            }
            Ok(_call_i_send(
                self.gateway_contract,
                self.env().transferred_value(),
                1u128,
                route_amount,
                route_recipient,
                dest_chain_id,
                request_metadata,
                request_packet,
            ))
        }

        #[ink(message, selector = 0x258404e2)]
        pub fn i_receive(
            &mut self,
            _request_sender: Bytes,
            packet: Bytes,
            src_chain_id: Bytes,
        ) -> InkVec<u8> {
            if self.env().caller() != self.gateway_contract {
                panic!("Only Gateway");
            }
            let src_chain_id = _get_utf8_bytes_to_string(&src_chain_id);
            let decoded_data = decode(&[ParamType::Uint(64), ParamType::String], &packet);
            if decoded_data.is_err() {
                panic!("Decode Error");
            }
            let decoded_data = decoded_data.unwrap();
            let nonce = decoded_data[0].clone().into_uint().unwrap().as_u64();
            let greeting = decoded_data[1].clone().into_string().unwrap();
            if greeting == String::from("Fail Dest Req") {
                panic!("Greeting == Fail Dest Req");
            }
            self.greeting_record
                .insert((src_chain_id, nonce), &greeting);
            packet
        }

        #[ink(message, selector = 0x9a50e5b4)]
        pub fn i_ack(
            &mut self,
            request_identifier: u128,
            exec_flag: bool,
            exec_data: InkVec<u8>,
        ) -> InkVec<u8> {
            if self.env().caller() != self.gateway_contract {
                panic!("Only Gateway");
            }
            if exec_flag {
                let decoded_data = decode(&[ParamType::Uint(64), ParamType::String], &exec_data);
                if decoded_data.is_err() {
                    panic!("Decode Error");
                }
                let decoded_data = decoded_data.unwrap();
                if decoded_data[1].clone().into_string().unwrap() == String::from("Fail Ack Req") {
                    panic!("Greeting == Fail Ack Req");
                }
            }
            self.ack_record_record.insert(request_identifier, &true);
            vec![]
        }

        #[ink(message)]
        pub fn get_greeting_record(&self, src_chain_id: Bytes, nonce: u64) -> Result<String> {
            let res = self
                .greeting_record
                .get((_get_utf8_bytes_to_string(&src_chain_id), nonce));
            if res.is_none() {
                return Err(TestError::NotFound);
            }
            Ok(res.unwrap())
        }

        #[ink(message)]
        pub fn get_ack_record(&self, request_identifier: u128) -> Result<bool> {
            let res = self.ack_record_record.get(request_identifier);
            if res.is_none() {
                return Err(TestError::NotFound);
            }
            Ok(res.unwrap())
        }

        #[ink(message)]
        pub fn get_route_token(&self) -> AccountId {
            self.route_token
        }

        #[ink(message)]
        pub fn get_contract_address(&self) -> AccountId {
            self.gateway_contract
        }
    }
}
