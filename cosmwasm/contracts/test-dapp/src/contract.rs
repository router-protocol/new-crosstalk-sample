use crate::execution::send_i_request;
use crate::query::handle_query;
use crate::state::{CREATE_I_SEND_REQUEST, PING_FROM_SOURCE, PONG_FROM_DESTINATION, REQUEST_ID};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::{from_binary, Coin, Event, Reply, StdError, SubMsgResult};
use new_crosstalk_sample::test_dapp::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use cw2::set_contract_version;

use router_wasm_bindings::ethabi::{decode, ParamType};
use router_wasm_bindings::types::CrosschainRequestResponse;
use router_wasm_bindings::{RouterMsg, RouterQuery, SudoMsg};

// version info for migration info
const CONTRACT_NAME: &str = "PingPong";
const CONTRACT_VERSION: &str = "0.1.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    REQUEST_ID.save(deps.storage, &0)?;
    Ok(Response::new().add_attribute("action", "ping_pong_init"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut<RouterQuery>, _env: Env, msg: Reply) -> StdResult<Response<RouterMsg>> {
    match msg.id {
        CREATE_I_SEND_REQUEST => {
            deps.api.debug(&msg.id.to_string());
            // TODO: need to handle nonce data here, Nonce handling logic depends on the use-case.
            let response: Response<RouterMsg> = Response::new();
            match msg.result {
                SubMsgResult::Ok(msg_result) => match msg_result.data {
                    Some(binary_data) => {
                        deps.api.debug("Binary Data Found");
                        let cross_chain_req_res: CrosschainRequestResponse =
                            from_binary(&binary_data).unwrap();

                        let info_str: String = format!(
                            "Binary data {:?}, response {:?}",
                            &binary_data.to_string(),
                            cross_chain_req_res
                        );
                        deps.api.debug(&info_str);
                        return Ok(response);
                    }
                    None => deps.api.debug("No Binary Data Found"),
                },
                SubMsgResult::Err(err) => deps.api.debug(&err.to_string()),
            }
        }
        id => return Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<RouterMsg>> {
    match msg {
        ExecuteMsg::SendIRequest {
            payload,
            dest_contract_address,
            dest_chain_id,
            request_metadata,
            amount,
            route_recipient,
        } => send_i_request(
            deps,
            info,
            payload,
            dest_contract_address,
            dest_chain_id,
            request_metadata,
            amount,
            route_recipient,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut<RouterQuery>, env: Env, msg: SudoMsg) -> StdResult<Response<RouterMsg>> {
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
            request_identifier,
            exec_flag,
            exec_data,
            refund_amount,
        } => handle_sudo_ack(
            deps,
            env,
            request_identifier,
            exec_flag,
            exec_data,
            refund_amount,
        ),
    }
}

pub fn handle_sudo_request(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    request_sender: String,
    src_chain_id: String,
    request_identifier: u64,
    payload: Binary,
) -> StdResult<Response<RouterMsg>> {
    let token_vec = match decode(&[ParamType::Uint(64), ParamType::String], &payload.0) {
        Ok(data) => data,
        Err(_) => {
            return Err(StdError::GenericErr {
                msg: String::from("err.into()"),
            })
        }
    };

    deps.api.debug("Inside the Inbound handler");
    let request_id: u64 = token_vec[0].clone().into_uint().unwrap().as_u64();
    let data_string: String = token_vec[1].clone().into_string().unwrap();

    if data_string.clone() == "Fail Dest Req".to_string() {
        return Err(StdError::GenericErr {
            msg: String::from("String != Fail Dest Req"),
        });
    }

    PING_FROM_SOURCE.save(deps.storage, (&src_chain_id, request_id), &data_string)?;

    let mut res = Response::new()
        .add_attribute("sender", request_sender)
        .add_attribute("request_identifier", request_identifier.to_string())
        .add_attribute("src_chain_id", src_chain_id);
    res.data = Some(payload);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut<RouterQuery>, env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME.to_string() {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }

    let info_str: String = format!(
        "migrating contract: {}, new_contract_version: {}, contract_name: {}",
        env.contract.address,
        CONTRACT_VERSION.to_string(),
        CONTRACT_NAME.to_string()
    );
    deps.api.debug(&info_str);
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<RouterQuery>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    handle_query(deps, env, msg)
}

fn handle_sudo_ack(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    _request_identifier: u64,
    exec_flag: bool,
    exec_data: Binary,
    _refund_amount: Coin,
) -> StdResult<Response<RouterMsg>> {
    let mut request_id: u64 = 0;
    if exec_flag {
        let token_vec = match decode(&[ParamType::Uint(64), ParamType::String], &exec_data.0) {
            Ok(data) => data,
            Err(_) => {
                return Err(StdError::GenericErr {
                    msg: String::from("err.into()"),
                })
            }
        };

        request_id = token_vec[0].clone().into_uint().unwrap().as_u64();
        let data_string: String = token_vec[1].clone().into_string().unwrap();

        if data_string.clone() == "Fail Ack Req".to_string() {
            return Err(StdError::GenericErr {
                msg: String::from("String != Fail Ack Req"),
            });
        }

        PONG_FROM_DESTINATION.save(deps.storage, &request_id.to_string(), &data_string)?;
    }

    let event = Event::new("ExecutionStatus")
        .add_attribute("requestIdentifier", request_id.to_string())
        .add_attribute("execFlag", exec_flag.to_string());

    Ok(Response::new().add_event(event))
}
