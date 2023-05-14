#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use cw0::parse_reply_instantiate_data;
use cw1155_base::msg::InstantiateMsg as Cw1155InstantiateMsg;

use cosmwasm_std::{to_binary, Reply, ReplyOn, SubMsg, WasmMsg};
use cw2::set_contract_version;
use router_wasm_bindings::{RouterMsg, RouterQuery, SudoMsg};

use crate::{
    execution::{handle_execute, handle_sudo},
    query::handle_query,
    state::{CW1155_CONTRACT_ADDRESS, OWNER},
};

use new_crosstalk_sample::xerc1155::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "xerc1155";
const CONTRACT_VERSION: &str = "1.0.0";

const INSTANTIATE_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.debug("Instantiating the contract🚀");

    // Store state with owner address
    OWNER.save(deps.storage, &info.sender.to_string())?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    //deploy cw1155-token
    Ok(Response::new().add_submessage(SubMsg {
        msg: WasmMsg::Instantiate {
            admin: Some(info.sender.to_string()),
            code_id: msg.xerc1155_codeid,
            msg: to_binary(&Cw1155InstantiateMsg {
                minter: env.contract.address.to_string(),
            })?,
            funds: vec![],
            label: "CW1155 TOKEN".to_string(),
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<RouterMsg>> {
    handle_execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut<RouterQuery>, env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let ver: cw2::ContractVersion = cw2::get_contract_version(deps.storage)?;
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    handle_query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut<RouterQuery>, env: Env, msg: SudoMsg) -> StdResult<Response<RouterMsg>> {
    handle_sudo(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut<RouterQuery>, _env: Env, msg: Reply) -> StdResult<Response<RouterMsg>> {
    match msg.id {
        INSTANTIATE_REPLY_ID => {
            // let data = msg.result.unwrap().data.unwrap();
            let response: cw0::MsgInstantiateContractResponse =
                parse_reply_instantiate_data(msg).unwrap();
            CW1155_CONTRACT_ADDRESS.save(deps.storage, &response.contract_address)?;
            return Ok(
                Response::new().add_attribute("cw20token", response.contract_address.clone())
            );
        }
        id => return Err(StdError::generic_err(format!("Unknown reply id: {}", id))),
    }
}
