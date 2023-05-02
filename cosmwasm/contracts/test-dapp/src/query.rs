use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw2::get_contract_version;
use new_crosstalk_sample::test_dapp::QueryMsg;
use router_wasm_bindings::RouterQuery;

use crate::state::{PING_FROM_SOURCE, PONG_FROM_DESTINATION};

pub fn handle_query(deps: Deps<RouterQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetContractVersion {} => to_binary(&get_contract_version(deps.storage)?),
        QueryMsg::FetchGreetingRecord {
            chain_id,
            request_id,
        } => to_binary(&fetch_ping(deps, &chain_id, request_id)?),
        QueryMsg::FetchAckRecord { request_id } => to_binary(&fetch_pong(deps, request_id)?),
    }
}

pub fn fetch_ping(deps: Deps<RouterQuery>, chain_id: &str, request_id: u64) -> StdResult<String> {
    PING_FROM_SOURCE.load(deps.storage, (chain_id, request_id))
}

pub fn fetch_pong(deps: Deps<RouterQuery>, request_id: u64) -> StdResult<String> {
    PONG_FROM_DESTINATION.load(deps.storage, &request_id.to_string())
}
