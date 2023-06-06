use cosmwasm_std::{Binary, DepsMut, MessageInfo, ReplyOn, Response, StdResult, SubMsg, Uint128, StdError};
use router_wasm_bindings::{
    ethabi::{encode, Token, ParamType, decode},
    Bytes, RouterMsg, RouterQuery,
};

use crate::state::CREATE_I_SEND_REQUEST;

pub fn send_i_request(
    deps: DepsMut<RouterQuery>,
    info: MessageInfo,
    payload: Binary,
    dest_contract_address: String,
    dest_chain_id: String,
    request_metadata: Binary,
    amount: Uint128,
    route_recipient: String,
) -> StdResult<Response<RouterMsg>> {
    let info_str: String = format!("create_outbound_request-- payload: {:?}", payload.clone(),);
    deps.api.debug(&info_str);

    if amount != Uint128::zero() {
        assert_eq!(info.funds.len(), 1);
        assert_eq!(info.funds[0].amount, amount);
    }
    let info_str: String = format!(
        "create_outbound_request-- dest_chain_id: {}, dest_contract_address: {}, request_metadata: {}",
        dest_chain_id, dest_contract_address.clone(), request_metadata
    );
    deps.api.debug(&info_str);
    let request_packet: Bytes = encode(&[
        Token::String(dest_contract_address.clone()),
        Token::Bytes(payload.clone().0),
    ]);

    let i_send_request: RouterMsg = RouterMsg::CrosschainCall {
        version: 1,
        route_amount: amount,
        route_recipient,
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

    let token_vec: Vec<Token> = match decode(&[ParamType::Uint(64), ParamType::String], &payload.0) {
        Ok(data) => data,
        Err(_) => {
            return Err(StdError::GenericErr {
                msg: String::from("err.into()"),
            })
        }
    };

    let greeting: String = token_vec[1].clone().into_string().unwrap();

    if greeting == "".to_string() {
        return  Err(StdError::GenericErr{
            msg: String::from("greeting cannot be empty")
        });
    }

    let res: Response<RouterMsg> = Response::new()
        .add_submessage(cross_chain_sub_msg.into())
        .add_attribute("dest_contract_address", dest_contract_address);
    Ok(res)
}
