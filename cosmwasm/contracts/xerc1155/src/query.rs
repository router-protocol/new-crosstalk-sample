use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw2::get_contract_version;
use new_crosstalk_sample::xerc1155::QueryMsg;

use crate::state::{CW1155_CONTRACT_ADDRESS, OWNER, REMOTE_CONTRACT_MAPPING};

pub fn handle_query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetContractVersion {} => to_binary(&get_contract_version(deps.storage)?),
        QueryMsg::GetOwner {} => to_binary(&get_owner(deps)?),
        QueryMsg::GetRemoteContract { chain_id } => {
            to_binary(&get_remote_contract(deps, chain_id)?)
        }
        QueryMsg::GetCw1155Address {} => to_binary(&get_cw1155_contract(deps)?),
    }
}

pub fn get_owner(deps: Deps) -> StdResult<String> {
    OWNER.load(deps.storage)
}

pub fn get_cw1155_contract(deps: Deps) -> StdResult<String> {
    CW1155_CONTRACT_ADDRESS.load(deps.storage)
}

fn get_remote_contract(deps: Deps, chain_id: String) -> StdResult<String> {
    REMOTE_CONTRACT_MAPPING.load(deps.storage, chain_id)
}
