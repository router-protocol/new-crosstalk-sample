use crate::contract::{execute, instantiate, query};
use crate::execution::{Cw721ExecuteMsg, Cw721QueryMsg};
use cw721::{ContractInfoResponse, NftInfoResponse, OwnerOfResponse};
use cw721_base::MintMsg;
use new_crosstalk_sample::xerc721::{ExecuteMsg, InstantiateMsg, QueryMsg, TransferParams};
use router_wasm_bindings::ethabi::{decode, ParamType, Token};
use router_wasm_bindings::types::RequestMetaData;
use router_wasm_bindings::RouterMsg;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{Binary, CosmosMsg, Deps, Empty, Env, MessageInfo, Response, StdError, Uint128};

use cosmwasm_std::from_binary;
use cosmwasm_std::DepsMut;
use cosmwasm_std::OwnedDeps;
use std::marker::PhantomData;
use std::vec;

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
        name: "XERC721".into(),
        symbol: "XERC721".into(),
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

fn get_nft_owner_of(deps: Deps, env: Env, token_id: String) -> Result<OwnerOfResponse, StdError> {
    let query_msg = Cw721QueryMsg::OwnerOf {
        token_id,
        include_expired: Some(false),
    };
    let owner_of = query(deps, env, query_msg);
    match owner_of {
        Ok(brr) => from_binary(&brr),
        Err(_) => Err(StdError::NotFound { kind: "nft".into() }),
    }
}

fn get_nft_info(
    deps: Deps,
    env: Env,
    token_id: String,
) -> Result<NftInfoResponse<Empty>, StdError> {
    let query_msg = Cw721QueryMsg::NftInfo { token_id };
    let nft_info = query(deps, env, query_msg);
    match nft_info {
        Ok(brr) => from_binary(&brr),
        Err(_) => Err(StdError::NotFound { kind: "nft".into() }),
    }
}

#[test]
fn test_basic() {
    let mut deps = get_mock_dependencies();
    let env = mock_env();

    do_instantiate(deps.as_mut());

    let msg = Cw721QueryMsg::ContractInfo {};
    let res = query(deps.as_ref(), env, msg).unwrap();
    let contract_info: ContractInfoResponse = from_binary(&res).unwrap();
    println!("{:?}", contract_info);
    assert_eq!(contract_info.name, "XERC721".to_string());
    assert_eq!(contract_info.symbol, "XERC721".to_string());
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
        token_uri: Some("someuri".to_string()),
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
        remote_contract.clone(),
    );

    let mint_msg = MintMsg {
        token_id: "2".into(),
        token_uri: Some("URI".to_string()),
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
    let response = get_nft_info(deps.as_ref(), env.clone(), "2".into());
    assert!(response.is_ok());

    let respone = get_nft_owner_of(deps.as_ref(), env.clone(), "2".into());
    assert!(response.is_ok());
    assert_eq!(respone.unwrap().owner, SENDER);

    let ext_cc_msg = ExecuteMsg::TransferCrossChain {
        dst_chain_id: "1".into(),
        token_id: 2,
        recipient: SENDER.to_string(),
        request_metadata,
    };
    let exec_msg = Cw721ExecuteMsg::Extension { msg: ext_cc_msg };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), exec_msg.clone());

    assert!(res.is_ok());

    if let Ok(result) = res {
        let _ok = match result.messages[0].msg.clone() {
            CosmosMsg::Custom(msg) => match msg {
                RouterMsg::CrosschainCall {
                    version: _,
                    route_amount: _,
                    route_recipient: _,
                    request_packet,
                    request_metadata: _,
                    dest_chain_id: _,
                } => {
                    // in order to verify encoded payload
                    // println!("{:?}", hex::encode(request_packet));
                    // let req_payload = hex::decode(hex::encode(request_packet)).unwrap();
                    // decode it as string bytes

                    let req_payload = Binary(request_packet);
                    let param_vec: Vec<ParamType> = vec![ParamType::String, ParamType::Bytes];
                    let token_vec = match decode(&param_vec, &req_payload.0) {
                        Ok(data) => data,
                        Err(_) => {
                            assert!(false);
                            vec![]
                        }
                    };
                    let dst_address = token_vec[0].clone().to_string();
                    assert!(dst_address == remote_contract);
                    let payload = token_vec[1].clone().into_bytes().unwrap();
                    test_decoding_in_ireceive(Binary(payload));

                    Ok(Response::<RouterMsg>::new())
                }
            },
            _ => Err(StdError::NotFound {
                kind: "isend".into(),
            }),
        };
    }
    // nft should be burned with id 2
    let response = get_nft_info(deps.as_ref(), env, "2".into());
    assert!(response.is_err());
}

fn test_decoding_in_ireceive(payload: Binary) {
    let params = TransferParams::get_params_types();
    let param_vec: Vec<ParamType> = vec![params];
    let token_vec = match decode(&param_vec, &payload.0) {
        Ok(data) => data,
        Err(_) => {
            assert!(false);
            return;
        }
    };
    let transfer_params_tokens: Vec<Token> = token_vec[0].clone().into_tuple().unwrap();
    let transfer_params: TransferParams =
        TransferParams::from_token_tuple(transfer_params_tokens).unwrap();

    assert!(transfer_params.nft_id == 2);
    assert!(transfer_params.recipient == SENDER.to_string());
}
