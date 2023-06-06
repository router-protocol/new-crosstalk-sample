use std::str::FromStr;

use ethabi::{encode, ethereum_types::U256, Token};
use near_sdk::{env, json_types::U128, AccountId};

use crate::contract::TestDapp;

#[test]
fn send_i_request_without_token() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let nonce: u64 = 1;
    let greeting: String = "hello".to_string();

    let nonce_u256: U256 = U256::from(nonce);
    let nonce_token: Token = Token::Uint(nonce_u256);
    let greeting_token: Token = Token::String(greeting);

    let payload: Vec<u8> = encode(&[nonce_token, greeting_token]);

    test_dapp.send_i_request(
        payload,
        "abcd".to_string(),
        "80001".to_string(),
        vec![1, 2, 3, 4],
        U128::from(0),
        "".to_string(),
    );
}

#[test]
#[should_panic(expected = "greeting cannot be empty")]
fn send_i_request_without_token_with_empty_string() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let nonce: u64 = 1;
    let greeting: String = "".to_string();

    let nonce_u256: U256 = U256::from(nonce);
    let nonce_token: Token = Token::Uint(nonce_u256);
    let greeting_token: Token = Token::String(greeting);

    let payload: Vec<u8> = encode(&[nonce_token, greeting_token]);

    test_dapp.send_i_request(
        payload,
        "abcd".to_string(),
        "80001".to_string(),
        vec![1, 2, 3, 4],
        U128::from(0),
        "".to_string(),
    );
}

#[test]
#[should_panic(expected = "Error in decoding payload: InvalidData")]
fn send_i_request_without_token_with_invalid_payload() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    test_dapp.send_i_request(
        vec![1, 1],
        "abcd".to_string(),
        "80001".to_string(),
        vec![1, 2, 3, 4],
        U128::from(0),
        "".to_string(),
    );
}

#[test]
fn send_i_request_with_token() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let nonce: u64 = 1;
    let greeting: String = "hello".to_string();

    let nonce_u256: U256 = U256::from(nonce);
    let nonce_token: Token = Token::Uint(nonce_u256);
    let greeting_token: Token = Token::String(greeting);

    let payload: Vec<u8> = encode(&[nonce_token, greeting_token]);

    test_dapp.send_i_request(
        payload,
        "abcd".to_string(),
        "80001".to_string(),
        vec![1, 2, 3, 4],
        U128::from(12),
        "Hello".to_string(),
    );
}

#[test]
#[should_panic(expected = "greeting cannot be empty")]
fn send_i_request_with_token_with_empty_string() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let nonce: u64 = 1;
    let greeting: String = "".to_string();

    let nonce_u256: U256 = U256::from(nonce);
    let nonce_token: Token = Token::Uint(nonce_u256);
    let greeting_token: Token = Token::String(greeting);

    let payload: Vec<u8> = encode(&[nonce_token, greeting_token]);

    test_dapp.send_i_request(
        payload,
        "abcd".to_string(),
        "80001".to_string(),
        vec![1, 2, 3, 4],
        U128::from(12),
        "Hello".to_string(),
    );
}

#[test]
#[should_panic(expected = "Error in decoding payload: InvalidData")]
fn send_i_request_with_token_with_invalid_payload() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    test_dapp.send_i_request(
        vec![1, 1],
        "abcd".to_string(),
        "80001".to_string(),
        vec![1, 2, 3, 4],
        U128::from(0),
        "".to_string(),
    );
}

#[test]
#[should_panic(expected = "Route recipient cannot be empty")]
fn send_i_request_with_token_with_empty_recipient() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let nonce: u64 = 1;
    let greeting: String = "".to_string();

    let nonce_u256: U256 = U256::from(nonce);
    let nonce_token: Token = Token::Uint(nonce_u256);
    let greeting_token: Token = Token::String(greeting);

    let payload: Vec<u8> = encode(&[nonce_token, greeting_token]);

    test_dapp.send_i_request(
        payload,
        "abcd".to_string(),
        "80001".to_string(),
        vec![1, 2, 3, 4],
        U128::from(12),
        "".to_string(),
    );
}

#[test]
fn test_i_receive() {
    let gateway: AccountId = env::predecessor_account_id();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let mut test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let request_id: u64 = 1;
    let request_id_u256 = ethabi::ethereum_types::U256::from(request_id);
    let request_id_token: Token = Token::Uint(request_id_u256);

    let greeting: String = "Hello".to_string();
    let greeting_token: Token = Token::String(greeting);

    let encoded_packet = ethabi::encode(&[request_id_token, greeting_token]);

    test_dapp.i_receive(
        "request_sender".to_string(),
        encoded_packet,
        "80001".to_string(),
    );

    let greeting: String = test_dapp.get_greeting_record("80001".to_string(), 1);
    assert_eq!(greeting, "Hello".to_string());
}

#[test]
#[should_panic(expected = "only gateway")]
fn test_i_receive_should_panic_wrong_gateway() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let mut test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    test_dapp.i_receive(
        "request_sender".to_string(),
        vec![0, 0, 0],
        "80001".to_string(),
    );
}

#[test]
#[should_panic(expected = "Error in decoding packet: InvalidData")]
fn test_i_receive_should_panic_wrong_decoding() {
    let gateway: AccountId = env::predecessor_account_id();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let mut test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    test_dapp.i_receive(
        "request_sender".to_string(),
        vec![0, 0, 0],
        "80001".to_string(),
    );
}

#[test]
#[should_panic(expected = "String != Fail Dest Req")]
fn test_i_receive_should_panic_failure_string() {
    let gateway: AccountId = env::predecessor_account_id();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let mut test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let request_id: u64 = 1;
    let request_id_u256: U256 = U256::from(request_id);
    let request_id_token: Token = Token::Uint(request_id_u256);

    let greeting: String = "Fail Dest Req".to_string();
    let greeting_token: Token = Token::String(greeting);

    let encoded_packet: Vec<u8> = ethabi::encode(&[request_id_token, greeting_token]);

    test_dapp.i_receive(
        "request_sender".to_string(),
        encoded_packet,
        "80001".to_string(),
    );
}

#[test]
fn test_i_ack() {
    let gateway: AccountId = env::predecessor_account_id();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let mut test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let request_id: u64 = 1;
    let request_id_u256: U256 = U256::from(request_id);
    let request_id_token: Token = Token::Uint(request_id_u256);

    let greeting: String = "hello".to_string();
    let greeting_token: Token = Token::String(greeting);

    let encoded_packet: Vec<u8> = ethabi::encode(&[request_id_token, greeting_token]);

    test_dapp.i_ack(U128::from(12), true, encoded_packet);

    let ack_record = test_dapp.get_ack_record(U128::from(12));
    assert_eq!(ack_record, true);
}

#[test]
#[should_panic(expected = "only gateway")]
fn test_i_ack_should_panic_only_gateway() {
    let gateway: AccountId = AccountId::from_str("gateway").unwrap();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let mut test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    test_dapp.i_ack(U128::from(12), true, vec![0, 0, 0]);
}

#[test]
#[should_panic(expected = "String != Fail Ack Req")]
fn test_i_ack_should_panic_failure_string() {
    let gateway: AccountId = env::predecessor_account_id();
    let route_token: AccountId = AccountId::from_str("route").unwrap();

    let mut test_dapp: TestDapp = TestDapp::new(gateway, route_token);

    let request_id: u64 = 1;
    let request_id_u256: U256 = U256::from(request_id);
    let request_id_token: Token = Token::Uint(request_id_u256);

    let greeting: String = "Fail Ack Req".to_string();
    let greeting_token: Token = Token::String(greeting);

    let encoded_packet: Vec<u8> = ethabi::encode(&[request_id_token, greeting_token]);

    test_dapp.i_ack(U128::from(12), true, encoded_packet);
}
