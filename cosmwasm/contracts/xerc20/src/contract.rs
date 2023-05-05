use crate::execution::handle_execute;
use crate::handle_sudo_execution::{handle_sudo_ack, handle_sudo_request};
use crate::query::handle_query;
use crate::state::{CREATE_I_SEND_REQUEST, CROSS_CHAIN_TOKEN, INSTANTIATE_REPLY_ID, OWNER};
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cosmwasm_std::{
    from_binary, to_binary, Reply, ReplyOn, StdError, SubMsg, SubMsgResult, WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw20::MinterResponse;
use new_crosstalk_sample::xerc20::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use cw2::set_contract_version;

use cw20_base::msg::InstantiateMsg as TokenInstantiateMsg;
use router_wasm_bindings::types::CrosschainRequestResponse;
use router_wasm_bindings::{RouterMsg, RouterQuery, SudoMsg};

// version info for migration info
const CONTRACT_NAME: &str = "XERC20";
const CONTRACT_VERSION: &str = "0.1.1";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<RouterQuery>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    OWNER.save(deps.storage, &info.sender.to_string())?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_submessage(SubMsg {
        // Create LP token
        msg: WasmMsg::Instantiate {
            admin: Some(info.sender.to_string()),
            code_id: msg.cw20_code_id,
            msg: to_binary(&TokenInstantiateMsg {
                name: msg.token_name,
                symbol: msg.token_symbol,
                decimals: 18,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
                marketing: None,
            })?,
            funds: vec![],
            label: "XERC20 TOKEN".to_string(),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut<RouterQuery>, _env: Env, msg: Reply) -> StdResult<Response<RouterMsg>> {
    match msg.id {
        INSTANTIATE_REPLY_ID => {
            // let data = msg.result.unwrap().data.unwrap();
            let response = parse_reply_instantiate_data(msg).unwrap();
            CROSS_CHAIN_TOKEN.save(deps.storage, &response.contract_address)?;
            return Ok(
                Response::new().add_attribute("cw20token", response.contract_address.clone())
            );
        }
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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<RouterMsg>> {
    handle_execute(deps, env, info, msg)
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut<RouterQuery>, env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME.to_string() {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    // if ver.version >= CONTRACT_VERSION.to_string() {
    //     return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    // }

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
