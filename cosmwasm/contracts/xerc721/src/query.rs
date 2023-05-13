use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw2::get_contract_version;
use new_crosstalk_sample::xerc721::QueryMsg;

use crate::{
    execution::{Cw721NFTContract, Cw721QueryMsg},
    state::{OWNER, REMOTE_CONTRACT_MAPPING},
};

pub fn handle_query(deps: Deps, env: Env, msg: Cw721QueryMsg) -> StdResult<Binary> {
    let tract = Cw721NFTContract::default();
    match msg {
        Cw721QueryMsg::Extension { msg } => match msg {
            QueryMsg::GetContractVersion {} => to_binary(&get_contract_version(deps.storage)?),
            QueryMsg::GetOwner {} => to_binary(&get_owner(deps)?),
            QueryMsg::GetRemoteContract { chain_id } => {
                to_binary(&get_remote_contract(deps, chain_id)?)
            }
        },
        _ => tract.query(deps, env, msg),
    }
}

pub fn get_owner(deps: Deps) -> StdResult<String> {
    OWNER.load(deps.storage)
}

fn get_remote_contract(deps: Deps, chain_id: String) -> StdResult<String> {
    REMOTE_CONTRACT_MAPPING.load(deps.storage, chain_id)
}
