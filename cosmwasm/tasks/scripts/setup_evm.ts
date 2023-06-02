import fs from "fs";
import dotenv from "dotenv";
import { Network, PrivateKey } from "@routerprotocol/router-chain-sdk-ts";
import { exec_msg } from "./execute_msg";
import { create_pair } from "./create_pair";
dotenv.config();

async function main() {
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

    const privateKey = PrivateKey.fromPrivateKey(privateKeyHash);
    const alice = privateKey.toBech32();

    const texchangeSetupFilePath = "config/texchange.json";
    const texchangeSetup = JSON.parse(fs.readFileSync(texchangeSetupFilePath, "utf-8"));
    
    const texchangeAddr = texchangeSetup[network]["texchange"]["addr"];
    if (!texchangeAddr) {
        throw new Error("Not able to find 'texchangeAddr' in texchange Setup file");
    }

    const wrappedMaticAddr = texchangeSetup[network]["wrappedMatic"]["addr"];
    if (!wrappedMaticAddr) {
        throw new Error("Not able to find 'wrappedMaticAddr' in texchange Setup file");
    }

    const wrappedAvaxAddr = texchangeSetup[network]["wrappedAvax"]["addr"];
    if (!wrappedAvaxAddr) {
        throw new Error("Not able to find 'wrappedAvaxAddr' in texchange Setup file");
    }

    const wrappedUsdcAddr = texchangeSetup[network]["wrappedUsdc"]["addr"];
    if (!wrappedUsdcAddr) {
        throw new Error("Not able to find 'wrappedUsdcAddr' in texchange Setup file");
    }


    let white_list_application_contract = {
        "chain_id": "80001",
        "chain_type": 1,
        "contract_address": "0xADA64Be2bC3C899Aa4791b7CDba82b910eC639D9"
    };
    let logs = await exec_msg(texchangeAddr, "white_list_application_contract", white_list_application_contract);
    console.log(logs);

    white_list_application_contract = {
        "chain_id": "43113",
        "chain_type": 1,
        "contract_address": "0xC305D1430BCb54388948f0cA1d5138f83DC7d97D"
    };
    logs = await exec_msg(texchangeAddr, "white_list_application_contract", white_list_application_contract);
    console.log(logs);
    
    let setSyntheticToken = {
        "contract_address": "0xC305D1430BCb54388948f0cA1d5138f83DC7d97D",
        "erc20_address": wrappedAvaxAddr,
        "dest_chain_id": "43113",
        "dest_chain_type": 1
    };
    logs = await exec_msg(texchangeAddr, "set_synthetic_token", setSyntheticToken);
    console.log(logs);

    setSyntheticToken = {
        "contract_address": "0xADA64Be2bC3C899Aa4791b7CDba82b910eC639D9",
        "erc20_address": wrappedUsdcAddr,
        "dest_chain_id": "80001",
        "dest_chain_type": 1
    };
    logs = await exec_msg(texchangeAddr, "set_synthetic_token", setSyntheticToken);
    console.log(logs);
    
}

main()
