use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, DepsMut, Env, Event, MessageInfo, ReplyOn, Response, StdResult,
    SubMsg, Uint128, WasmMsg,
};
use new_crosstalk_sample::xerc20::{ChainTypeInfo, ContractInfo, ExecuteMsg};
use router_wasm_bindings::{
    ethabi::{encode, ethereum_types::U256, Token},
    types::{AckType, RequestMetaData},
    Bytes, RouterMsg, RouterQuery,
};

use crate::{
    modifiers::is_owner_modifier,
    query::{fetch_oracle_gas_price, fetch_white_listed_contract},
    state::{
        CHAIN_ID, CHAIN_TYPE_MAPPING, CREATE_I_SEND_REQUEST, CROSS_CHAIN_TOKEN,
        WHITELISTED_CONTRACT_MAPPING,
    },
};

pub fn handle_execute(
    deps: DepsMut<RouterQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response<RouterMsg>> {
    match msg {
        ExecuteMsg::SetChainTypes { chain_type_info } => {
            set_chain_types_info(deps, env, info, chain_type_info)
        }
        ExecuteMsg::SetChainId { id } => set_chain_id(deps, env, info, id),
        ExecuteMsg::SetXerc20Addr { addr } => set_xerc20_addr(deps, env, info, addr),
        ExecuteMsg::SetWhiteListedContracts { contracts } => {
            set_white_listed_contracts(deps, &env, &info, contracts)
        }
        ExecuteMsg::TrasferCrossChain {
            amount,
            recipient,
            dest_chain_id,
        } => transfer_cross_chain(deps, env, info, amount, recipient, dest_chain_id),
    }
}

pub fn set_white_listed_contracts(
    deps: DepsMut<RouterQuery>,
    _env: &Env,
    info: &MessageInfo,
    contracts: Vec<ContractInfo>,
) -> StdResult<Response<RouterMsg>> {
    is_owner_modifier(deps.as_ref(), &info)?;

    for i in 0..contracts.len() {
        WHITELISTED_CONTRACT_MAPPING.save(
            deps.storage,
            &contracts[i].chain_id,
            &contracts[i].contract_addr,
        )?;
    }

    let res = Response::new().add_attribute("action", "SetCustodyContracts");
    Ok(res)
}

/**
 * @notice Used to set chain type info operations of the given chain (chainId, chainType).
 * @notice Only callable by Admin.
 * @param  chain_type_info   chain infos (chain_id & chain_type)

*/
pub fn set_chain_types_info(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    info: MessageInfo,
    chain_type_info: Vec<ChainTypeInfo>,
) -> StdResult<Response<RouterMsg>> {
    is_owner_modifier(deps.as_ref(), &info)?;

    for i in 0..chain_type_info.len() {
        CHAIN_TYPE_MAPPING.save(
            deps.storage,
            &chain_type_info[i].chain_id,
            &chain_type_info[i].chain_type,
        )?;
    }
    let event_name: String = String::from("SetChainTypeInfo");
    let set_chain_bytes_info_event: Event = Event::new(event_name);

    let res = Response::new()
        .add_attribute("action", "SetChainTypeInfo")
        .add_event(set_chain_bytes_info_event);
    Ok(res)
}

pub fn set_chain_id(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    info: MessageInfo,
    id: String,
) -> StdResult<Response<RouterMsg>> {
    is_owner_modifier(deps.as_ref(), &info)?;

    CHAIN_ID.save(deps.storage, &id)?;
    let event_name: String = String::from("SetChainId");
    let event: Event = Event::new(event_name);

    let res = Response::new()
        .add_attribute("action", "SetChainId")
        .add_event(event);
    Ok(res)
}

/**
 * @notice Used to set xerc20 token address
 * @notice Only callable by Admin.
 * @param  addr   xerc20 token address

*/
pub fn set_xerc20_addr(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    info: MessageInfo,
    addr: String,
) -> StdResult<Response<RouterMsg>> {
    is_owner_modifier(deps.as_ref(), &info)?;

    CROSS_CHAIN_TOKEN.save(deps.storage, &addr)?;
    let event_name: String = String::from("SetXERC20Addr");
    let event: Event = Event::new(event_name);

    let res = Response::new()
        .add_attribute("action", "SetXERC20Addr")
        .add_event(event);
    Ok(res)
}

pub fn transfer_cross_chain(
    deps: DepsMut<RouterQuery>,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Binary,
    dest_chain_id: String,
) -> StdResult<Response<RouterMsg>> {
    let u256: U256 = U256::from(amount.u128());
    let payload: Vec<u8> = encode(&[Token::Bytes(recipient.0), Token::Uint(u256)]);
    let burn_msg = cw20_base::msg::ExecuteMsg::BurnFrom {
        owner: info.sender.to_string(),
        amount,
    };

    let xerc20_token: String = CROSS_CHAIN_TOKEN.load(deps.storage)?;
    let exec_burn_msg: CosmosMsg<RouterMsg> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: xerc20_token,
        funds: vec![],
        msg: to_binary(&burn_msg)?,
    });
    let chain_id: String = CHAIN_ID.load(deps.storage)?;
    let ack_gas_price: u64 = fetch_oracle_gas_price(deps.as_ref(), chain_id)?;
    let dest_gas_price: u64 = fetch_oracle_gas_price(deps.as_ref(), dest_chain_id.clone())?;
    let dest_contract_address: String = fetch_white_listed_contract(deps.as_ref(), &dest_chain_id)?;
    let request_metadata: RequestMetaData = RequestMetaData {
        dest_gas_limit: 200_000,
        dest_gas_price,
        ack_gas_limit: 200_000,
        ack_gas_price,
        relayer_fee: Uint128::zero(),
        ack_type: AckType::AckOnBoth,
        is_read_call: false,
        asm_address: String::default(),
    };
    let info_str: String = format!(
        "create_outbound_request-- dest_chain_id: {}, dest_contract_address: {}, request_metadata: {:?}",
        dest_chain_id, dest_contract_address.clone(), request_metadata
    );
    deps.api.debug(&info_str);
    let request_packet: Bytes = encode(&[
        Token::String(dest_contract_address.clone()),
        Token::Bytes(payload),
    ]);

    let i_send_request: RouterMsg = RouterMsg::CrosschainCall {
        version: 1,
        route_amount: Uint128::zero(),
        route_recipient: String::default(),
        dest_chain_id,
        request_metadata: request_metadata.get_abi_encoded_bytes(),
        request_packet,
    };
    let cross_chain_sub_msg: SubMsg<RouterMsg> = SubMsg {
        id: CREATE_I_SEND_REQUEST,
        msg: i_send_request.into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };
    let res = Response::new()
        .add_message(exec_burn_msg)
        .add_submessage(cross_chain_sub_msg.into())
        .add_attribute("dest_contract_address", dest_contract_address);
    Ok(res)
}
