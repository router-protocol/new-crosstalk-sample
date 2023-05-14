use crate::{Deserialize, Serialize};
use cosmwasm_std::{Binary, CustomMsg, StdResult, Uint128};
use router_wasm_bindings::{
    ethabi::{ethereum_types::U256, ParamType, Token},
    types::{ChainType, RequestMetaData},
    utils::{convert_address_from_bytes_to_string, convert_address_from_string_to_bytes},
    Bytes,
};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub minter: String, // fee payer will be contract itself
    pub xerc1155_codeid: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferParams {
    pub nft_ids: Vec<Uint128>,
    pub nft_amounts: Vec<Uint128>,
    pub nft_data: Bytes,
    pub recipient: String,
}

impl TransferParams {
    pub fn get_evm_encoding(&self) -> StdResult<Token> {
        let nft_ids_len = self.nft_ids.clone().len();
        let nft_amounts_len = self.nft_ids.clone().len();
        assert!(nft_ids_len == nft_amounts_len); // both length should be equal

        let mut nft_ids = Vec::<Token>::new();
        for i in 0..nft_ids_len {
            nft_ids.push(Token::Uint(U256::from(Uint128::u128(
                &self.nft_ids.clone()[i],
            ))));
        }
        let nft_ids_array = Token::FixedArray(nft_ids);

        let mut nft_amounts = Vec::<Token>::new();
        for i in 0..nft_amounts_len {
            nft_amounts.push(Token::Uint(U256::from(Uint128::u128(
                &self.nft_amounts.clone()[i],
            ))));
        }
        let nft_amounts_array = Token::FixedArray(nft_amounts);
        let nft_data = Token::Bytes(self.nft_data.clone());
        let recipient = Token::Bytes(self.recipient.clone().into());

        Ok(Token::Tuple(vec![
            nft_ids_array,
            nft_amounts_array,
            nft_data,
            recipient,
        ]))
    }
    pub fn get_params_types() -> ParamType {
        return ParamType::Tuple(vec![
            ParamType::Array(Box::new(ParamType::Uint(256))),
            ParamType::Array(Box::new(ParamType::Uint(256))),
            ParamType::Bytes,
            ParamType::Bytes,
        ]);
    }
    pub fn from_token_tuple(tuple: Vec<Token>) -> StdResult<Self> {
        let nfts_ids_token = tuple[0].clone().into_array().unwrap();
        let mut nft_ids = Vec::<Uint128>::new();
        for i in 0..nfts_ids_token.len() {
            nft_ids.push(Uint128::from(
                nfts_ids_token[i].clone().into_uint().unwrap().as_u128(),
            ));
        }

        let nft_amounts_token = tuple[1].clone().into_array().unwrap();
        let mut nft_amounts = Vec::<Uint128>::new();
        for i in 0..nft_amounts_token.len() {
            nft_amounts.push(Uint128::from(
                nft_amounts_token[i].clone().into_uint().unwrap().as_u128(),
            ));
        }

        let nft_data = tuple[2].clone().into_bytes().unwrap();
        let has_prefix = true;
        let prefix = if has_prefix { "0x" } else { "" };
        let recipient = format!(
            "{}{}",
            prefix,
            tuple[3]
                .clone()
                .into_bytes()
                .unwrap()
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        );
        Ok(Self {
            nft_data,
            recipient,
            nft_amounts,
            nft_ids,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    EnrollRemoteContract {
        chain_id: String,
        remote_address: String,
    },
    SetCw1155ContractAddress {
        address: String,
    },
    TransferCrossChain {
        dst_chain_id: String,
        token_ids: Vec<Uint128>,
        token_amounts: Vec<Uint128>,
        token_data: Binary,
        recipient: String,
        request_metadata: RequestMetaData,
    },
    BatchMint {
        to: String,
        batch: Vec<(String, Uint128)>,
        msg: String,
    },
    Mint {
        to: String,
        token_id: Uint128,
        amount: Uint128,
        msg: String,
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
    GetCw1155Address {},
}

impl CustomMsg for QueryMsg {}
