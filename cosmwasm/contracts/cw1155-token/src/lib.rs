use cosmwasm_std::StdError;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw1155_base::msg::InstantiateMsg;

use cw1155::{Cw1155ExecuteMsg, Cw1155QueryMsg};
use cw1155_base::contract::{
    execute as cw1155_execute, instantiate as cw1155_instantiate, query as cw1155_query,
};
use cw1155_base::ContractError;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    cw1155_instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw1155ExecuteMsg,
) -> Result<Response, ContractError> {
    cw1155_execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: Cw1155QueryMsg) -> StdResult<Binary> {
    cw1155_query(deps, env, msg)
}
