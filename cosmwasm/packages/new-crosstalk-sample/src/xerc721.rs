use crate::{Deserialize, Serialize};
use cosmwasm_std::{CustomMsg, StdResult};
use router_wasm_bindings::{
    ethabi::{ethereum_types::U256, ParamType, Token},
    types::RequestMetaData,
};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String, // fee payer will be contract itself
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransferParams {
    pub nft_id: u64,
    pub recipient: String,
}

impl TransferParams {
    pub fn get_evm_encoding(&self) -> StdResult<Token> {
        let token_id = Token::Uint(U256::from(self.nft_id.clone()));
        let recipient = Token::Bytes(self.clone().recipient.into_bytes());
        Ok(Token::Tuple(vec![token_id, recipient]))
    }
    pub fn get_params_types() -> ParamType {
        return ParamType::Tuple(vec![ParamType::Uint(256), ParamType::Bytes]);
    }
    pub fn from_token_tuple(tuple: Vec<Token>) -> StdResult<Self> {
        let nft_id = tuple[0].clone().into_uint().unwrap().as_u64();
        let recipient = tuple[1].clone().into_string().unwrap();
        Ok(Self { nft_id, recipient })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    EnrollRemoteContract {
        chain_id: String,
        remote_address: String,
    },
    TransferCrossChain {
        dst_chain_id: String,
        token_id: u64,
        recipient: String,
        request_metadata: RequestMetaData,
    },
}

impl CustomMsg for ExecuteMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // fetch contract version
    GetContractVersion {},
    GetOwner {},
    GetRemoteContract { chain_id: String },
}

impl CustomMsg for QueryMsg {}
