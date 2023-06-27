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
    let data = hex
        ::decode(
            "00000000002dc6c0000000000000000000002d79883d20000000000000000000000000000000000000000000000000000100"
        )
        .unwrap();
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
fn abi_decode_request_packet() {
    let request_packet: Vec<u8> = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,64,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,192,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,65,114,111,117,116,101,114,49,100,110,121,48,55,106,118,110,107,101,109,118,53,115,110,100,118,97,119,110,48,120,117,114,120,121,53,55,112,55,119,104,110,57,103,112,120,109,102,115,104,113,56,53,114,108,99,116,115,122,103,115,115,113,120,122,54,104,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,128,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,96,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,71,13,228,223,130,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,160,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,64,123,153,57,15,28,109,145,134,104,105,58,54,185,111,122,226,72,163,79,108,44,63,32,248,200,126,4,239,177,24,163,165,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,34,117,115,100,99,45,50,46,114,111,117,116,101,114,112,114,111,116,111,99,111,108,97,108,112,104,97,46,116,101,115,116,110,101,116,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,27,114,111,117,116,101,114,112,114,111,116,111,99,111,108,97,108,112,104,97,46,116,101,115,116,110,101,116,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,34,117,115,100,99,45,50,46,114,111,117,116,101,114,112,114,111,116,111,99,111,108,97,108,112,104,97,46,116,101,115,116,110,101,116,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,96,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,160,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,224,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,32,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,64,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,96,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,20,15,168,120,26,131,228,104,38,98,27,59,192,148,234,42,2,18,231,27,35,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,20,15,168,120,26,131,228,104,38,98,27,59,192,148,234,42,2,18,231,27,35,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,20,237,254,224,46,252,203,197,161,149,181,13,233,50,219,118,3,132,203,128,69,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];

    let tokens = decode(
        &[
            ParamType::String,
            ParamType::Bytes,
        ],
        &request_packet
    ).unwrap();

    let packet: Vec<u8> = tokens[1].clone().into_bytes().unwrap();

    let decoded = decode(&[ParamType::Uint(64)], &packet).unwrap();
    println!("{:?}", decoded);
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
