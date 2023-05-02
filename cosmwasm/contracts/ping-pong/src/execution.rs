use cosmwasm_std::{Binary, DepsMut, ReplyOn, Response, StdResult, SubMsg, Uint128};
use router_wasm_bindings::{
    ethabi::{encode, ethereum_types::U256, Token},
    Bytes, RouterMsg, RouterQuery,
};

use crate::{
    query::fetch_request_id,
    state::{CREATE_I_SEND_REQUEST, REQUEST_ID},
};

pub fn i_ping(
    deps: DepsMut<RouterQuery>,
    ping: String,
    dest_contract_address: String,
    dest_chain_id: String,
    request_metadata: Binary,
) -> StdResult<Response<RouterMsg>> {
    let request_id: u64 = fetch_request_id(deps.as_ref())? + 1;
    let u256: U256 = U256::from(request_id);
    REQUEST_ID.save(deps.storage, &(request_id))?;
    let payload: Vec<u8> = encode(&[Token::Uint(u256), Token::String(ping)]);
    let info_str: String = format!("create_outbound_request-- payload: {:?}", payload.clone(),);
    deps.api.debug(&info_str);
    let route_amount: Uint128 = Uint128::zero();

    let info_str: String = format!(
        "create_outbound_request-- dest_chain_id: {}, dest_contract_address: {}, request_metadata: {}",
        dest_chain_id, dest_contract_address.clone(), request_metadata
    );
    deps.api.debug(&info_str);
    let request_packet: Bytes = encode(&[
        Token::String(dest_contract_address.clone()),
        Token::Bytes(payload),
    ]);

    let i_send_request: RouterMsg = RouterMsg::CrosschainCall {
        version: 1,
        route_amount,
        route_recipient: String::default(),
        dest_chain_id,
        request_metadata: request_metadata.0,
        request_packet,
    };
    let cross_chain_sub_msg: SubMsg<RouterMsg> = SubMsg {
        id: CREATE_I_SEND_REQUEST,
        msg: i_send_request.into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };
    let res = Response::new()
        .add_submessage(cross_chain_sub_msg.into())
        .add_attribute("dest_contract_address", dest_contract_address);
    Ok(res)
}
