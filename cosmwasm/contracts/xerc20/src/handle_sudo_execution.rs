use cosmwasm_std::{
    to_binary, Binary, Coin, CosmosMsg, DepsMut, Env, Event, Response, StdError, StdResult,
    Uint128, WasmMsg,
};
use router_wasm_bindings::{
    ethabi::{decode, ParamType},
    types::ChainType,
    utils::convert_address_from_bytes_to_string,
    Bytes, RouterMsg, RouterQuery,
};

use crate::{
    modifiers::is_white_listed_modifier,
    state::{CHAIN_TYPE_MAPPING, CROSS_CHAIN_TOKEN},
};

pub fn handle_sudo_request(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    request_sender: String,
    src_chain_id: String,
    request_identifier: u64,
    payload: Binary,
) -> StdResult<Response<RouterMsg>> {
    deps.api.debug("XERC20 INFO: Handle Sudo Request");
    let src_chain_type: u64 = CHAIN_TYPE_MAPPING.load(deps.storage, &src_chain_id)?;
    let sender: String = match src_chain_type {
        1 => request_sender.to_lowercase(),
        _ => request_sender.clone(),
    };

    is_white_listed_modifier(deps.as_ref(), &src_chain_id, &sender)?;
    deps.api.debug("Request Coming from whitelisted Contract");
    // bytes memory packet = abi.encode(recipient, amount);
    let token_vec = match decode(&[ParamType::Bytes, ParamType::Uint(128)], &payload.0) {
        Ok(data) => data,
        Err(_) => {
            return Err(StdError::GenericErr {
                msg: String::from("err.into()"),
            });
        }
    };

    let u128_val: u128 = token_vec[1].clone().into_uint().unwrap().as_u128();
    let amount = Uint128::new(u128_val);
    let addr: Bytes = token_vec[0].clone().into_bytes().unwrap();

    let recipient =
        convert_address_from_bytes_to_string(&addr, ChainType::ChainTypeCosmos.get_chain_code())?;
    let info_str: String = format!("recipient {:?}, amount {:?}", recipient, amount);
    deps.api.debug(&info_str);

    deps.api.addr_validate(&recipient)?;
    let mint_msg = cw20_base::msg::ExecuteMsg::Mint { recipient, amount };

    let xerc20_token: String = CROSS_CHAIN_TOKEN.load(deps.storage)?;
    let exec_mint_msg: CosmosMsg<RouterMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: xerc20_token,
        funds: vec![],
        msg: to_binary(&mint_msg)?,
    });
    let info_str: String = format!("exec_mint_token {:?}", exec_mint_msg);
    deps.api.debug(&info_str);

    let res: Response<RouterMsg> = Response::new()
        .add_message(exec_mint_msg)
        .add_attribute("sender", request_sender)
        .add_attribute("request_identifier", request_identifier.to_string())
        .add_attribute("src_chain_id", src_chain_id);
    Ok(res)
}

pub fn handle_sudo_ack(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    request_identifier: u64,
    exec_flag: bool,
    exec_data: Binary,
    _refund_amount: Coin,
) -> StdResult<Response<RouterMsg>> {
    let info_str: String = format!(
        "handle_sudo_ack, request_identifier {:?}, exec_data {:?}",
        request_identifier, exec_data
    );
    deps.api.debug(&info_str);
    let event = Event::new("ExecutionStatus")
        .add_attribute("requestIdentifier", request_identifier.to_string())
        .add_attribute("execFlag", exec_flag.to_string());

    Ok(Response::new().add_event(event))
}
