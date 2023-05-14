use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, WasmMsg,
};
use cw1155::{Cw1155ExecuteMsg, TokenId};
use new_crosstalk_sample::xerc1155::{ExecuteMsg, TransferParams};
use router_wasm_bindings::{
    ethabi::{decode, encode, ParamType, Token},
    types::RequestMetaData,
    Bytes, RouterMsg, RouterQuery, SudoMsg,
};

use crate::state::{CW1155_CONTRACT_ADDRESS, OWNER, REMOTE_CONTRACT_MAPPING};

pub fn handle_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<RouterMsg>> {
    match msg {
        ExecuteMsg::EnrollRemoteContract {
            chain_id,
            remote_address,
        } => enroll_remote_contract(deps, env, info, chain_id, remote_address),
        ExecuteMsg::SetCw1155ContractAddress { address } => {
            set_cw1155_contract(deps, env, info, address)
        }
        ExecuteMsg::BatchMint { to, batch, msg } => batch_mint(deps, env, info, to, batch, msg),
        ExecuteMsg::Mint {
            to,
            token_id,
            amount,
            msg,
        } => mint(deps, env, info, to, token_id, amount, msg),

        ExecuteMsg::TransferCrossChain {
            dst_chain_id,
            token_ids,
            token_amounts,
            token_data,
            recipient,
            request_metadata,
        } => transfer_crosschain(
            deps,
            env,
            info,
            dst_chain_id,
            token_ids,
            token_amounts,
            token_data,
            recipient,
            request_metadata,
        ),
    }
}

pub fn only_owner(deps: Deps, info: MessageInfo) -> StdResult<Response<RouterMsg>> {
    if info.sender.to_string() != OWNER.load(deps.storage).unwrap() {
        return Err(StdError::GenericErr {
            msg: "Auth: Invalid Owner".into(),
        });
    } else {
        Ok(Response::new())
    }
}

pub fn enroll_remote_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    chain_id: String,
    remote_address: String,
) -> StdResult<Response<RouterMsg>> {
    only_owner(deps.as_ref(), info)?;
    REMOTE_CONTRACT_MAPPING.save(deps.storage, chain_id, &remote_address)?;
    Ok(Response::new())
}

pub fn batch_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    to: String,
    batch: Vec<(String, Uint128)>,
    msg: String,
) -> StdResult<Response<RouterMsg>> {
    only_owner(deps.as_ref(), info)?;
    let mint_msg = Cw1155ExecuteMsg::BatchMint {
        to: deps.api.addr_validate(&to).unwrap().to_string(),
        batch,
        msg: Some(to_binary(&msg).unwrap()),
    };
    let exec_mint_msg: CosmosMsg<RouterMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: CW1155_CONTRACT_ADDRESS.load(deps.storage).unwrap(),
        funds: vec![],
        msg: to_binary(&mint_msg)?,
    });
    Ok(Response::new().add_message(exec_mint_msg))
}

pub fn mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    to: String,
    token_id: Uint128,
    amount: Uint128,
    msg: String,
) -> StdResult<Response<RouterMsg>> {
    only_owner(deps.as_ref(), info)?;
    let mint_msg = Cw1155ExecuteMsg::Mint {
        to: deps.api.addr_validate(&to).unwrap().to_string(),
        token_id: token_id.to_string(),
        value: amount,
        msg: Some(to_binary(&msg).unwrap()),
    };
    let exec_mint_msg: CosmosMsg<RouterMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: CW1155_CONTRACT_ADDRESS.load(deps.storage).unwrap(),
        funds: vec![],
        msg: to_binary(&mint_msg)?,
    });
    Ok(Response::new().add_message(exec_mint_msg))
}

pub fn set_cw1155_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: String,
) -> StdResult<Response<RouterMsg>> {
    only_owner(deps.as_ref(), info)?;
    CW1155_CONTRACT_ADDRESS.save(deps.storage, &address)?;
    Ok(Response::new())
}

pub fn transfer_crosschain(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    dst_chain_id: String,
    token_ids: Vec<Uint128>,
    token_amounts: Vec<Uint128>,
    token_data: Binary,
    recipient: String,
    request_metadata: RequestMetaData,
) -> StdResult<Response<RouterMsg>> {
    let mut batch: Vec<(TokenId, Uint128)> = Vec::new();
    assert_eq!(token_ids.len(), token_amounts.len());
    for i in 0..token_ids.len() {
        batch.push((token_ids[i].to_string(), token_amounts[i]));
    }
    let burn_msg = Cw1155ExecuteMsg::BatchBurn {
        from: info.sender.into_string(),
        batch,
    };
    // burn will itself verify if user have enough token or owner of token or not
    let exec_burn_msg: CosmosMsg<RouterMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: CW1155_CONTRACT_ADDRESS.load(deps.storage).unwrap(),
        funds: vec![],
        msg: to_binary(&burn_msg)?,
    });

    let dst_contract_add: String = REMOTE_CONTRACT_MAPPING
        .load(deps.storage, dst_chain_id.clone())
        .unwrap();

    let transfer_params = TransferParams {
        nft_ids: token_ids,
        nft_amounts: token_amounts,
        nft_data: token_data.into(),
        recipient,
    };

    let encoded_payload: Vec<u8> = encode(&[transfer_params.get_evm_encoding()?]);
    let request_packet: Bytes = encode(&[
        Token::String(dst_contract_add),
        Token::Bytes(encoded_payload),
    ]);

    let i_send_request: RouterMsg = RouterMsg::CrosschainCall {
        version: 1,
        route_amount: Uint128::new(0u128),
        route_recipient: String::from(""),
        dest_chain_id: dst_chain_id.clone(),
        request_metadata: request_metadata.get_abi_encoded_bytes(),
        request_packet,
    };

    Ok(Response::new().add_messages([
        exec_burn_msg,
        cosmwasm_std::CosmosMsg::Custom(i_send_request),
    ]))
}

pub fn handle_sudo(
    deps: DepsMut<RouterQuery>,
    env: Env,
    msg: SudoMsg,
) -> StdResult<Response<RouterMsg>> {
    match msg {
        SudoMsg::HandleIReceive {
            request_sender,
            src_chain_id,
            request_identifier,
            payload,
        } => handle_sudo_request(
            deps,
            env,
            request_sender,
            src_chain_id,
            request_identifier,
            payload,
        ),
        SudoMsg::HandleIAck {
            request_identifier: _,
            exec_flag: _,
            exec_data: _,
            refund_amount: _,
        } => Ok(Response::new()),
    }
}

pub fn handle_sudo_request(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    request_sender: String,
    src_chain_id: String,
    _request_identifier: u64,
    payload: Binary,
) -> StdResult<Response<RouterMsg>> {
    let remote_contract_add = REMOTE_CONTRACT_MAPPING
        .load(deps.storage, src_chain_id.clone())
        .unwrap();

    if remote_contract_add != request_sender {
        return Err(StdError::GenericErr {
            msg: "Invalid sender".to_string(),
        });
    }

    let params = TransferParams::get_params_types();
    let param_vec: Vec<ParamType> = vec![params];
    let token_vec = match decode(&param_vec, &payload.0) {
        Ok(data) => data,
        Err(_) => {
            return Err(StdError::GenericErr {
                msg: String::from("err.into()"),
            })
        }
    };
    let transfer_params_tokens: Vec<Token> = token_vec[0].clone().into_tuple().unwrap();
    let transfer_params: TransferParams = TransferParams::from_token_tuple(transfer_params_tokens)?;
    let mut batch: Vec<(TokenId, Uint128)> = Vec::new();
    for i in 0..transfer_params.nft_ids.len() {
        batch.push((
            transfer_params.nft_ids[i].to_string(),
            transfer_params.nft_amounts[i],
        ));
    }

    let mint_msg = Cw1155ExecuteMsg::BatchMint {
        to: deps
            .api
            .addr_validate(&transfer_params.recipient)
            .unwrap()
            .to_string(),
        batch,
        msg: Some(to_binary(&transfer_params.nft_data).unwrap()),
    };
    let exec_mint_msg: CosmosMsg<RouterMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: CW1155_CONTRACT_ADDRESS.load(deps.storage).unwrap(),
        funds: vec![],
        msg: to_binary(&mint_msg)?,
    });

    let mut response = Response::new();
    let encoded_ack_payload: Vec<u8> = encode(&[Token::String(src_chain_id)]);
    response.data = Some(Binary(encoded_ack_payload));
    Ok(response.add_message(exec_mint_msg))
}
