#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use router_wasm_bindings::{RouterMsg, RouterQuery, SudoMsg};

use crate::{
    execution::{handle_execute, handle_sudo, Cw721ExecuteMsg, Cw721NFTContract, Cw721QueryMsg},
    query::handle_query,
    state::OWNER,
};
use cw721_base::msg::InstantiateMsg as Cw721InstantiateMsg;

use new_crosstalk_sample::xerc721::{InstantiateMsg, MigrateMsg};

// version info for migration info
const CONTRACT_NAME: &str = "xerc721";
const CONTRACT_VERSION: &str = "1.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    deps.api.debug("Instantiating the contract🚀");

    // Store state with owner address
    OWNER.save(deps.storage, &info.sender.to_string())?;

    let tract = Cw721NFTContract::default();

    let minter = deps.api.addr_validate(&msg.minter)?;
    let msg = Cw721InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        minter: minter.to_string(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    tract.instantiate(deps.branch(), env.clone(), info.clone(), msg)?;

    Ok(Response::new().add_attribute("action", "xcw721-init"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ExecuteMsg,
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
pub fn query(deps: Deps, env: Env, msg: Cw721QueryMsg) -> StdResult<Binary> {
    handle_query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut<RouterQuery>, env: Env, msg: SudoMsg) -> StdResult<Response<RouterMsg>> {
    handle_sudo(deps, env, msg)
}
