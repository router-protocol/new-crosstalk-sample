use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult, Uint128};
use cw2::get_contract_version;
use new_crosstalk_sample::xerc20::QueryMsg;
use router_wasm_bindings::{
    types::{GasPriceResponse, TokenPriceResponse},
    RouterQuerier, RouterQuery,
};

use crate::state::{
    CHAIN_ID, CHAIN_TYPE_MAPPING, CROSS_CHAIN_TOKEN, OWNER, WHITELISTED_CONTRACT_MAPPING,
};

pub fn handle_query(deps: Deps<RouterQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetContractVersion {} => to_binary(&get_contract_version(deps.storage)?),
        QueryMsg::FetchWhiteListedContract { chain_id } => {
            to_binary(&fetch_white_listed_contract(deps, &chain_id)?)
        }
        QueryMsg::FetchOwner {} => to_binary(&fetch_owner(deps)?),
        QueryMsg::FetchXerc20 {} => to_binary(&fetch_xerc20_addr(deps)?),
        QueryMsg::FetchChainId {} => to_binary(&fetch_chain_id(deps)?),
        QueryMsg::AllWhiteListedContract {} => to_binary(&fetch_all_white_listed_contract(deps)?),
        QueryMsg::FetchChainType { chain_id } => to_binary(&fetch_chain_type(deps, &chain_id)?),
    }
}

pub fn is_white_listed_contract(
    deps: Deps<RouterQuery>,
    chain_id: &str,
    contract_addr: &str,
) -> bool {
    match WHITELISTED_CONTRACT_MAPPING.load(deps.storage, chain_id) {
        Ok(contract) => return contract == contract_addr,
        _ => false,
    }
}

pub fn fetch_white_listed_contract(deps: Deps<RouterQuery>, chain_id: &str) -> StdResult<String> {
    WHITELISTED_CONTRACT_MAPPING.load(deps.storage, chain_id)
}

pub fn fetch_owner(deps: Deps<RouterQuery>) -> StdResult<String> {
    OWNER.load(deps.storage)
}

pub fn fetch_all_white_listed_contract(
    deps: Deps<RouterQuery>,
) -> StdResult<Vec<(String, String)>> {
    match WHITELISTED_CONTRACT_MAPPING
        .range(deps.storage, None, None, Order::Ascending)
        .collect()
    {
        Ok(data) => return Ok(data),
        Err(err) => return Err(err),
    };
}

/**
 * @notice Used to fetch chain_info.
 * @param   chain_id
*/
pub fn fetch_chain_type(deps: Deps<RouterQuery>, chain_id: &str) -> StdResult<u64> {
    CHAIN_TYPE_MAPPING.load(deps.storage, chain_id)
}

pub fn fetch_oracle_gas_price(deps: Deps<RouterQuery>, chain_id: String) -> StdResult<u64> {
    let router_querier: RouterQuerier = RouterQuerier::new(&deps.querier);
    let gas_price_response: GasPriceResponse = router_querier.gas_price(chain_id)?;
    Ok((gas_price_response.gas_price * 120) / 100)
}

pub fn fetch_oracle_token_price(deps: Deps<RouterQuery>, symbol: String) -> StdResult<Uint128> {
    let router_querier: RouterQuerier = RouterQuerier::new(&deps.querier);
    let token_price_response: TokenPriceResponse = router_querier.token_price(symbol)?;
    Ok(token_price_response.token_price)
}

pub fn fetch_xerc20_addr(deps: Deps<RouterQuery>) -> StdResult<String> {
    CROSS_CHAIN_TOKEN.load(deps.storage)
}

pub fn fetch_chain_id(deps: Deps<RouterQuery>) -> StdResult<String> {
    CHAIN_ID.load(deps.storage)
}
