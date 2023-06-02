use crate::*;
use near_sdk::{env, json_types::U128};
use router_wasm_bindings::ethabi::{decode, encode, ethereum_types::U256, ParamType, Token};

#[test]
fn test_ping_to_dest() {
    let gateway: AccountId = env::predecessor_account_id();
    let mut ping_pong: PingPong = PingPong::new(gateway.clone());

    let dest_gas_limit: u64 = 1000000;
    let dest_gas_price: u64 = 100000000000;
    let ack_gas_limit: u64 = 100000000000000;
    let ack_gas_price: u64 = 1000000000;
    let relayer_fees: U128 = U128::from(1000000000000000);
    let ack_type: u8 = 1;
    let is_read_call: bool = false;
    let asm_address: String = "".to_string();

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

    ping_pong.i_ping(
        "80001".to_string(),
        "shivam.near".to_string(),
        "hello".to_string(),
        request_metadata,
        "shivam".to_string()
    );
}

#[test]
fn test_ping_from_source() {
    let gateway: AccountId = env::predecessor_account_id();
    let mut ping_pong: PingPong = PingPong::new(gateway.clone());

    let message: String = String::from("hello");

    let dest_gas_limit: u64 = 1000000;
    let dest_gas_price: u64 = 100000000000;
    let ack_gas_limit: u64 = 100000000000000;
    let ack_gas_price: u64 = 1000000000;
    let relayer_fees: U128 = U128::from(1000000000000000);
    let ack_type: u8 = 1;
    let is_read_call: bool = false;
    let asm_address: String = "".to_string();

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

    ping_pong.i_ping(
        "80001".to_string(),
        "shivam.near".to_string(),
        "hello".to_string(),
        request_metadata,
        "shivam".to_string()
    );

    let request_id: u64 = ping_pong.get_current_request_id();

    let request_id_token: Token = Token::Uint(U256::from(request_id.clone()));
    let message_token: Token = Token::String(message.clone());
    let packet: Vec<u8> = encode(&[request_id_token, message_token]);

    ping_pong.i_receive(
        "hello".to_string(),
        packet,
        "80001".to_string(),
    );

    let ping_from_source = ping_pong.get_ping_from_source("80001".to_string(), request_id);
    assert_eq!(ping_from_source, message);
}

#[test]
fn test_crosstalk_ack() {
    let gateway: AccountId = env::predecessor_account_id();
    let mut ping_pong: PingPong = PingPong::new(gateway.clone());

    let message: String = String::from("hello");
    let request_id: u64 = 1;

    let request_id_token: Token = Token::Uint(U256::from(request_id.clone()));
    let message_token: Token = Token::String(message.clone());
    let payload: Vec<u8> = encode(&[request_id_token, message_token]);

    let request_identifier: U128 = U128::from(1);
    let exec_flags: bool = true;
    let exec_data: Vec<u8> = payload;

    ping_pong.i_ack(
        request_identifier,
        exec_flags,
        exec_data,
    );

    let ack_from_destination = ping_pong.get_ack_from_destination(request_id);
    assert_eq!(ack_from_destination, message);
}

#[test]
fn get_request_metadata() {
    let dest_gas_limit: u64 = 1000000;
    let dest_gas_price: u64 = 100000000000;
    let ack_gas_limit: u64 = 100000000000000;
    let ack_gas_price: u64 = 1000000000;
    let relayer_fees: U128 = U128::from(1000000000000000);
    let ack_type: u8 = 1;
    let is_read_call: bool = false;
    let asm_address: String = "".to_string();

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

    println!("Request Metadata: {:?}", request_metadata);
}

#[test]
fn abi_decode() {
    // let data = hex
    //     ::decode(
    //         "00005af3107a4000000000003b9aca0000000000000000000000000000000000000000000000000000038d7ea4c680000000"
    //     )
    //     .unwrap();
    // println!("{:?}", data);
    let data = hex::encode(vec![
        91, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44,
        48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48,
        44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 54, 44, 48, 44, 48, 44,
        48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48,
        44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44,
        48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 54, 52, 44, 48, 44, 48, 44, 48, 44, 48, 44,
        48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48,
        44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44,
        48, 44, 48, 44, 48, 44, 48, 44, 49, 52, 44, 55, 50, 44, 49, 48, 49, 44, 49, 48, 56, 44, 49,
        48, 56, 44, 49, 49, 49, 44, 51, 50, 44, 56, 51, 44, 49, 48, 52, 44, 57, 55, 44, 49, 49, 53,
        44, 49, 48, 52, 44, 49, 49, 56, 44, 57, 55, 44, 49, 49, 54, 44, 48, 44, 48, 44, 48, 44, 48,
        44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44, 48, 44,
        48, 44, 48, 44, 48, 93,
    ]);
    println!("{:?}", data);
    // let param_vec: Vec<ParamType> = vec![ParamType::String, ParamType::Bool];

    // let token_vec: Vec<Token> = match decode(&param_vec, &data) {
    //     Ok(data) => data,
    //     Err(_) => env::panic_str("not able to decode the payload"),
    // };

    // let hello_string: String = token_vec[0].clone().into_string().unwrap();
    // let to_revert: bool = token_vec[1].clone().into_bool().unwrap();

    // println!("hello: {}", hello_string);
    // println!("To revert: {}", to_revert);
}

#[test]
fn abi_decode_with_array() {
    let sample_vec: Vec<u8> = vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    println!(
        "{:?}",
        decode(
            &[
                ParamType::Array(Box::new(ParamType::Bool)),
                ParamType::Array(Box::new(ParamType::Bytes)),
            ],
            &sample_vec
        )
    );
}

#[test]
fn hex_decode() {
    let data = hex
        ::decode(
            "00000000000F4240000000003B9ACA0000000000000000000000000000000000000000000000000000000000000000000000"
        )
        .unwrap();
    println!("{:?}", data);
}
