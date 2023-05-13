use crate::contract::{execute, instantiate, query};
use crate::execution::{Cw721ExecuteMsg, Cw721QueryMsg};
use cw721::NftInfoResponse;
use cw721_base::MintMsg;
use new_crosstalk_sample::xerc721::TransferParams;
use new_crosstalk_sample::xerc721::{ExecuteMsg, InstantiateMsg, QueryMsg};
use router_wasm_bindings::types::RequestMetaData;
use router_wasm_bindings::RouterMsg;
use serde_json;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{CosmosMsg, Empty, Env, MessageInfo, Uint128};

use cosmwasm_std::from_binary;
use cosmwasm_std::DepsMut;
use cosmwasm_std::OwnedDeps;
use std::marker::PhantomData;

const SENDER: &str = "router1apapk9zfz3rp4x87fsm6h0s3zd0wlmkz0fx8tx";

fn get_mock_dependencies() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData::default(),
    }
}

fn do_instantiate(mut deps: DepsMut) {
    let instantiate_msg = InstantiateMsg {
        name: "ERC721".into(),
        symbol: "ERC721".into(),
        minter: SENDER.to_string(),
    };
    let info = mock_info(SENDER, &[]);
    let env = mock_env();
    let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
}

fn set_remote_contract(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    chain_id: String,
    remote_contract: String,
) {
    let extension_msg = ExecuteMsg::EnrollRemoteContract {
        chain_id: chain_id.clone(),
        remote_address: remote_contract.clone(),
    };
    let enroll_msg = cw721_base::ExecuteMsg::Extension { msg: extension_msg };
    let res = execute(deps, env.clone(), _info, enroll_msg.clone());
    assert!(res.is_ok());
}

#[test]
fn test_basic() {
    let mut deps = get_mock_dependencies();
    do_instantiate(deps.as_mut());
}

#[test]
fn test_enroll_and_get_remote_contract() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(SENDER, &[]);
    let chain_id = "cosmos".to_string();
    let remote_contract = "wasm1kjd9yyyqx0jwfzzy9ls32vuuyfem38x2lg2y0g".to_string();

    do_instantiate(deps.as_mut());

    set_remote_contract(
        deps.as_mut(),
        env.clone(),
        info,
        chain_id.clone(),
        remote_contract.clone(),
    );
    // Get remote contract
    let extension_msg = QueryMsg::GetRemoteContract { chain_id };
    let query_msg = Cw721QueryMsg::Extension { msg: extension_msg };
    let res = query(deps.as_ref(), env.clone(), query_msg.clone());
    let remote_contract_result: String = from_binary(&res.unwrap()).unwrap();

    // Check if remote contract is set
    assert_eq!(remote_contract_result, remote_contract);
}

#[test]
fn test_mint_nft() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(SENDER, &[]);

    do_instantiate(deps.as_mut());

    let mint_msg = MintMsg {
        token_id: "2".into(),
        token_uri: None,
        owner: SENDER.into(),
        extension: Empty {},
    };
    let mint_msg = Cw721ExecuteMsg::Mint(mint_msg);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), mint_msg.clone());
    assert!(res.is_ok());
}

#[test]
fn test_transfer_crosschain() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info(SENDER, &[]);
    let remote_contract = "wasm1kjd9yyyqx0jwfzzy9ls32vuuyfem38x2lg2y0g".to_string();

    do_instantiate(deps.as_mut());
    set_remote_contract(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        "1".into(),
        remote_contract,
    );

    let mint_msg = MintMsg {
        token_id: "2".into(),
        token_uri: None,
        owner: SENDER.into(),
        extension: Empty {},
    };
    let mint_msg = Cw721ExecuteMsg::Mint(mint_msg);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), mint_msg.clone());
    assert!(res.is_ok());

    let request_metadata: RequestMetaData = RequestMetaData {
        dest_gas_limit: 0,
        ack_gas_limit: 0,
        dest_gas_price: 0,
        ack_gas_price: 0,
        relayer_fee: Uint128::from(0u32),
        ack_type: router_wasm_bindings::types::AckType::AckOnBoth,
        is_read_call: false,
        asm_address: "".into(),
    };
    let ext_cc_msg = ExecuteMsg::TransferCrossChain {
        dst_chain_id: "1".into(),
        token_id: 2,
        recipient: SENDER.to_string(),
        request_metadata,
    };
    let exec_msg = Cw721ExecuteMsg::Extension { msg: ext_cc_msg };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), exec_msg.clone());
    assert!(res.is_ok());
}
