//

use std::marker::PhantomData;
use std::vec;

use crate::contract::instantiate;
use crate::contract::{execute, sudo};
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    DepsMut,
};
use cosmwasm_std::{Binary, Coin, OwnedDeps, Uint128};
use new_crosstalk_sample::xerc20::{ChainTypeInfo, ExecuteMsg, InstantiateMsg};
use router_wasm_bindings::ethabi::{decode, ParamType};
use router_wasm_bindings::types::{AckType, ChainType, RequestMetaData};
use router_wasm_bindings::utils::{
    convert_address_from_bytes_to_string, convert_address_from_string_to_bytes,
};
use router_wasm_bindings::{Bytes, RouterQuery, SudoMsg};

const INIT_ADDRESS: &str = "router1apapk9zfz3rp4x87fsm6h0s3zd0wlmkz0fx8tx";
const BRIDGE_ADDRESS: &str = "0xeedb3ab68d567a6cd6d19fa819fe77b9f8ed1538";

fn do_instantiate(mut deps: DepsMut<RouterQuery>) {
    let instantiate_msg = InstantiateMsg {
        cw20_code_id: 1,
        token_name: String::from("sdsdsds"),
        token_symbol: String::from("dsdwdw"),
    };
    let info = mock_info(INIT_ADDRESS, &[]);
    let env = mock_env();
    let res = instantiate(deps.branch(), env.clone(), info.clone(), instantiate_msg).unwrap();
    assert_eq!(1, res.messages.len());

    let chain_types: ExecuteMsg = ExecuteMsg::SetChainTypes {
        chain_type_info: vec![
            ChainTypeInfo {
                chain_id: "80001".to_string(),
                chain_type: 1,
            },
            ChainTypeInfo {
                chain_id: "5".to_string(),
                chain_type: 1,
            },
            ChainTypeInfo {
                chain_id: "43113".to_string(),
                chain_type: 1,
            },
            ChainTypeInfo {
                chain_id: "router_9000-1".to_string(),
                chain_type: 2,
            },
        ],
    };
    execute(deps.branch(), env.clone(), info.clone(), chain_types).unwrap();
    let set_xerc20: ExecuteMsg = ExecuteMsg::SetXerc20Addr {
        addr: INIT_ADDRESS.to_string(),
    };
    execute(deps.branch(), env.clone(), info, set_xerc20).unwrap();
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
    let binary: Binary = Binary::from_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAN4Lazp2QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABBcm91dGVyMXJkbDdmZGp0azRjc3JmenBzc2ptenMwMzhyejd1bWRjbm03bmVkdDNoamNkZjc0enI5N3FqMm03ZmMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap();
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

// #[test]
// fn test_execute_create_outbound_request() {
//     let mut deps = OwnedDeps {
//         storage: MockStorage::default(),
//         api: MockApi::default(),
//         querier: MockQuerier::default(),
//         custom_query_type: PhantomData,
//     };
//     do_instantiate(deps.as_mut());
//     let env = mock_env();
//     let greeting: String = String::from("Hello Lord Venky");

//     let msg: ExecuteMsg = ExecuteMsg::IPing {
//         dest_contract_address: String::from(BRIDGE_ADDRESS),
//         dest_chain_id: String::from("80001"),
//         ping: greeting,
//         request_metadata: Binary(vec![]),
//     };
//     let info = mock_info(INIT_ADDRESS, &[]);
//     let response = execute(deps.as_mut(), env, info, msg).unwrap();
//     assert_eq!(response.messages.len(), 1);

//     let message = response.messages.get(0).unwrap();
//     let router_msg = message.msg.clone();
//     match router_msg {
//         CosmosMsg::Custom(msg) => match msg {
//             RouterMsg::CrosschainCall {
//                 version,
//                 route_amount,
//                 route_recipient,
//                 dest_chain_id,
//                 request_metadata,
//                 request_packet,
//             } => {
//                 assert_eq!(route_amount, Uint128::zero());
//                 assert_eq!(route_recipient, "");

//                 assert_eq!(dest_chain_id, "80001");
//                 assert_eq!(version, 1);
//                 assert_eq!(hex::encode(request_metadata), "");
//                 assert_eq!(hex::encode(request_packet), "000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a30786565646233616236386435363761366364366431396661383139666537376239663865643135333800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001048656c6c6f204c6f72642056656e6b7900000000000000000000000000000000");
//             }
//         },
//         _ => {}
//     }
// }

#[test]
fn test_encode() {
    let addr: String =
        String::from("0xEeDb3AB68d567A6CD6D19Fa819fe77b9f8Ed1538");
    println!(
        "{:?}",
        Binary(
            convert_address_from_string_to_bytes(addr, ChainType::ChainTypeEvm.get_chain_code())
                .unwrap()
        ).to_base64()
    );

    let binary: Binary = Binary::from_base64("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAN4Lazp2QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABBcm91dGVyMXJkbDdmZGp0azRjc3JmenBzc2ptenMwMzhyejd1bWRjbm03bmVkdDNoamNkZjc0enI5N3FqMm03ZmMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA").unwrap();
    println!(
        "{:?}",
        decode(&[ParamType::Bytes, ParamType::Uint(128)], &binary.0)
    );
    let cc: Bytes = vec![
        114, 111, 117, 116, 101, 114, 49, 114, 100, 108, 55, 102, 100, 106, 116, 107, 52, 99, 115,
        114, 102, 122, 112, 115, 115, 106, 109, 122, 115, 48, 51, 56, 114, 122, 55, 117, 109, 100,
        99, 110, 109, 55, 110, 101, 100, 116, 51, 104, 106, 99, 100, 102, 55, 52, 122, 114, 57, 55,
        113, 106, 50, 109, 55, 102, 99,
    ];
    println!(
        "{:?}",
        convert_address_from_bytes_to_string(&cc, ChainType::ChainTypeCosmos.get_chain_code())
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
    let bin: Binary = Binary(rm.get_abi_encoded_bytes());
    println!("{:?}", hex::encode(bin.0));
}
