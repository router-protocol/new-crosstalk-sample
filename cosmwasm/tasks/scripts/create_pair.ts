import fs from "fs";
import dotenv from "dotenv";
import { init_wasm_code } from "./init_contract";
import { upload_wasm_code } from "./upload_wasm";
import { Network, PrivateKey } from "@routerprotocol/router-chain-sdk-ts";
import { exec_msg } from "./execute_msg";
dotenv.config();

export const create_pair = async function (routerSwapFactoryAddr: string, addr1: string, addr2: string, name1: string, name2: string): Promise<(string[])> {
    let network = Network.AlphaDevnet;
    if (process.env.NETWORK == "devnet") {
        network = Network.Devnet
    } else if (process.env.NETWORK == "testnet") {
        network = Network.Testnet
    } else if (process.env.NETWORK == "mainnet") {
        network = Network.Mainnet
    } else if (process.env.NETWORK && process.env.NETWORK != "alpha-devnet") {
        throw new Error("Please set your NETWORK in the .env file");
    }

    const privateKeyHash = process.env.PRIVATE_KEY;
    const chainId = process.env.CHAIN_ID;
    if (!chainId) {
        throw new Error("Please set your CHAIN_ID in the .env file");
    }

    if (!privateKeyHash) {
        throw new Error("Please set your PRIVATE_KEY in the .env file");
    }


    type Token = {
        contract_addr: string;
    }
    type AssetInfo = {
        token: Token;
    }
    type PairConfig = {
        asset_infos: AssetInfo[];
        token_name: string;
        token_symbol: string;
        token_decimal: number;
    };
    const token1: Token = {
        contract_addr: addr1
    }
    const token2: Token = {
        contract_addr: addr2
    }
    const assetInfo1: AssetInfo = {
        token: token1
    }
    const assetInfo2: AssetInfo = {
        token: token2
    }
    const createPairInitMsg: PairConfig = {
    asset_infos: [assetInfo1, assetInfo2],
    token_name: name1 + name2 + "pair",
    token_symbol: name1 + name2 + "pair",
    token_decimal: 18
    };
    console.log(JSON.stringify(createPairInitMsg));
    
    const createPairLogs = await exec_msg(routerSwapFactoryAddr, "create_pair", createPairInitMsg);
    
    let pair_contract_addr: string = "";
    let liquidity_token_addr: string = "";
    for (let i = 0; i < createPairLogs[0].events.length; i++) {
        if (createPairLogs[0].events[i].type == "wasm") {
            const attrs = createPairLogs[0].events[i].attributes;
            for (let j = 0; j < attrs.length; j++) {
                if (attrs[j].key == "pair_contract_addr") {
                    pair_contract_addr = attrs[j].value;
                }
                if (attrs[j].key == "liquidity_token_addr") {
                    liquidity_token_addr = attrs[j].value;
                }
            }
        }
    }

    return [pair_contract_addr, liquidity_token_addr];
}

