use std::marker::PhantomData;

use crate::contract::instantiate;
use crate::contract::{execute, sudo};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    DepsMut,
};
use cosmwasm_std::{Binary, Coin, CosmosMsg, OwnedDeps, Uint128};
use new_crosstalk_sample::ping_pong::{ExecuteMsg, InstantiateMsg};
use router_wasm_bindings::ethabi::{decode, ParamType};
use router_wasm_bindings::types::{AckType, ChainType, RequestMetaData};
use router_wasm_bindings::utils::convert_address_from_string_to_bytes;
use router_wasm_bindings::{Bytes, RouterMsg, RouterQuery, SudoMsg};

const INIT_ADDRESS: &str = "router1apapk9zfz3rp4x87fsm6h0s3zd0wlmkz0fx8tx";
const BRIDGE_ADDRESS: &str = "0xeedb3ab68d567a6cd6d19fa819fe77b9f8ed1538";

fn do_instantiate(mut deps: DepsMut<RouterQuery>) {
    let instantiate_msg = InstantiateMsg {};
    let info = mock_info(INIT_ADDRESS, &[]);
    let env = mock_env();
    let res = instantiate(deps.branch(), env.clone(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn test_basic() {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    };
    do_instantiate(deps.as_mut());
}

#[test]
fn test_sudo_inbound_function() {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    };
    do_instantiate(deps.as_mut());
    let env = mock_env();
    let payload: Bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000a68656c6c6f2070696e6700000000000000000000000000000000000000000000").unwrap();
    let binary: Binary = Binary(payload);
    let msg: SudoMsg = SudoMsg::HandleIReceive {
        request_sender: BRIDGE_ADDRESS.into(),
        src_chain_id: String::from("80001"),
        request_identifier: 2,
        payload: binary,
    };

    let result = sudo(deps.as_mut(), env, msg);
    if result.is_err() {
        println!("{:?}", result.as_ref().err());
        assert!(false);
        return;
    }
    let response = result.unwrap();
    assert_eq!(response.messages.len(), 0);
}

#[test]
fn test_sudo_outbound_ack_function() {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    };
    do_instantiate(deps.as_mut());
    let env = mock_env();

    let payload: Bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000a68656c6c6f2070696e6700000000000000000000000000000000000000000000").unwrap();
    let binary_data: Binary = Binary(payload);
    let msg: SudoMsg = SudoMsg::HandleIAck {
        request_identifier: 1,
        exec_flag: true,
        exec_data: binary_data,
        refund_amount: Coin::new(123u128, String::from("route")),
    };

    let result = sudo(deps.as_mut(), env, msg);
    if result.is_err() {
        println!("{:?}", result.as_ref().err());
        assert!(false);
        return;
    }
    let response = result.unwrap();
    assert_eq!(response.messages.len(), 0);
}

#[test]
fn test_execute_create_outbound_request() {
    let mut deps = OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    };
    do_instantiate(deps.as_mut());
    let env = mock_env();
    let greeting: String = String::from("Hello Lord Venky");

    let msg: ExecuteMsg = ExecuteMsg::IPing {
        dest_contract_address: String::from(BRIDGE_ADDRESS),
        dest_chain_id: String::from("80001"),
        ping: greeting,
        request_metadata: Binary(vec![]),
    };
    let info = mock_info(INIT_ADDRESS, &[]);
    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(response.messages.len(), 1);

    let message = response.messages.get(0).unwrap();
    let router_msg = message.msg.clone();
    match router_msg {
        CosmosMsg::Custom(msg) => match msg {
            RouterMsg::CrosschainCall {
                version,
                route_amount,
                route_recipient,
                dest_chain_id,
                request_metadata,
                request_packet,
            } => {
                assert_eq!(route_amount, Uint128::zero());
                assert_eq!(route_recipient, "");

                assert_eq!(dest_chain_id, "80001");
                assert_eq!(version, 1);
                assert_eq!(hex::encode(request_metadata), "");
                assert_eq!(hex::encode(request_packet), "000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a30786565646233616236386435363761366364366431396661383139666537376239663865643135333800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001048656c6c6f204c6f72642056656e6b7900000000000000000000000000000000");
            }
        },
        _ => {}
    }
}

#[test]
fn test_encode() {
    let addr: String =
        String::from("router1ghd753shjuwexxywmgs4xz7x2q732vcnkm6h2pyv9s6ah3hylvrq8h5484");
    println!(
        "{:?}",
        hex::encode(
            convert_address_from_string_to_bytes(addr, ChainType::ChainTypeCosmos.get_chain_code())
                .unwrap()
        )
    );

    let binary: Binary = Binary::from_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKSGVsbG8gTG9yZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap();
    println!(
        "{:?}",
        decode(&[ParamType::Uint(64), ParamType::String], &binary.0)
    );

    let rm: RequestMetaData = RequestMetaData {
        dest_gas_limit: 200_000,
        dest_gas_price: 50_000_000_000,
        ack_gas_limit: 200_000,
        ack_gas_price: 50_000_000_000,
        relayer_fee: Uint128::zero(),
        ack_type: AckType::AckOnBoth,
        is_read_call: false,
        asm_address: String::default(),
    };
    println!("{:?}", Binary(rm.get_abi_encoded_bytes()).to_base64());
}
