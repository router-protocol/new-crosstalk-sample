use cosmwasm_std::{
    Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult, Uint128,
};
use cw721_base::{state::TokenInfo, Cw721Contract};
use new_crosstalk_sample::xerc721::{ExecuteMsg, QueryMsg, TransferParams};
// use rand::Rng;
use router_wasm_bindings::{
    ethabi::{decode, encode, ParamType, Token},
    types::RequestMetaData,
    Bytes, RouterMsg, RouterQuery, SudoMsg,
};

use crate::state::{OWNER, REMOTE_CONTRACT_MAPPING};
pub type Cw721NFTContract<'a> = Cw721Contract<'a, Empty, Empty, ExecuteMsg, QueryMsg>;
pub type Cw721ExecuteMsg = cw721_base::ExecuteMsg<Empty, ExecuteMsg>;
pub type Cw721QueryMsg = cw721_base::QueryMsg<QueryMsg>;

pub fn handle_execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ExecuteMsg,
) -> StdResult<Response<RouterMsg>> {
    match msg {
        Cw721ExecuteMsg::Extension { msg } => match msg {
            ExecuteMsg::EnrollRemoteContract {
                chain_id,
                remote_address,
            } => enroll_remote_contract(deps, env, info, chain_id, remote_address),
            ExecuteMsg::TransferCrossChain {
                dst_chain_id,
                token_id,
                recipient,
                request_metadata,
            } => transfer_crosschain(
                deps,
                env,
                info,
                dst_chain_id,
                token_id,
                recipient,
                request_metadata,
            ),
        },
        _ => match Cw721NFTContract::default().execute(deps, env, info, msg) {
            Ok(cw721_res) => {
                let response: Response<RouterMsg> = Response::<RouterMsg>::new()
                    .add_attributes(cw721_res.attributes)
                    .add_events(cw721_res.events);
                return Ok(response);
            }
            Err(err) => {
                return Err(StdError::GenericErr {
                    msg: err.to_string(),
                })
            }
        },
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

pub fn transfer_crosschain(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    dst_chain_id: String,
    token_id: u64,
    recipient: String,
    request_metadata: RequestMetaData,
) -> StdResult<Response<RouterMsg>> {
    let tract = Cw721NFTContract::default();
    let rider_info = tract.tokens.load(deps.storage, &token_id.to_string())?;
    match tract.check_can_send(deps.as_ref(), &_env, &info, &rider_info) {
        Err(_) => {
            return Err(StdError::GenericErr {
                msg: "ContractError::Unauthorized".to_string(),
            })
        }
        _ => (),
    }

    // burn nft
    tract.tokens.remove(deps.storage, &token_id.to_string())?;
    tract.decrement_tokens(deps.storage)?;

    let dst_contract_add: String = REMOTE_CONTRACT_MAPPING
        .load(deps.storage, dst_chain_id.clone())
        .unwrap();

    let transfer_params = TransferParams {
        recipient,
        nft_id: token_id,
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

    Ok(Response::new().add_message(i_send_request))
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

    // mint nft
    let tract = Cw721NFTContract::default();
    let token_info = TokenInfo {
        owner: deps.api.addr_validate(&transfer_params.recipient)?,
        approvals: vec![],
        token_uri: None,
        extension: Empty {},
    };
    tract.tokens.save(
        deps.storage,
        &transfer_params.nft_id.to_string(),
        &token_info,
    )?;
    tract.increment_tokens(deps.storage)?;

    let mut response = Response::new();
    let encoded_ack_payload: Vec<u8> = encode(&[Token::String(src_chain_id)]);
    response.data = Some(Binary(encoded_ack_payload));
    Ok(response)
}
