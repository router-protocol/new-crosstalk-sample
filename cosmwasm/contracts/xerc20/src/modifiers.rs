use cosmwasm_std::{Deps, MessageInfo, StdError, StdResult};
use router_wasm_bindings::RouterQuery;

use crate::{query::is_white_listed_contract, state::OWNER};

pub fn is_owner_modifier(deps: Deps<RouterQuery>, info: &MessageInfo) -> StdResult<()> {
    let owner: String = match OWNER.load(deps.storage) {
        Ok(owner) => owner,
        Err(err) => return StdResult::Err(err),
    };
    if owner != info.sender {
        return StdResult::Err(StdError::GenericErr {
            msg: String::from("Auth: Invalid Owner"),
        });
    }
    Ok(())
}

pub fn is_white_listed_modifier(
    deps: Deps<RouterQuery>,
    chain_id: &str,
    contract: &str,
) -> StdResult<()> {
    let is_white_listed_contract = is_white_listed_contract(deps, chain_id, contract);
    let info_str: String = format!("--chain_id: {:?}, contract: {:?}", chain_id, contract);
    deps.api.debug(&info_str);
    if !is_white_listed_contract {
        let info_str: String = format!(
            "Auth: The Sender/Receiver contract is not whitelisted, chain_id: {:?}, contract: {:?}",
            chain_id, contract
        );
        deps.api.debug(&info_str);
        return StdResult::Err(StdError::GenericErr { msg: info_str });
    }
    Ok(())
}
