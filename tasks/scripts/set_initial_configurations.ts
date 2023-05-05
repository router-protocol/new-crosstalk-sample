import fs from "fs";
import dotenv from "dotenv";
import { init_wasm_code } from "./init_contract";
import { upload_wasm_code } from "./upload_wasm";
import { Network, PrivateKey } from "@routerprotocol/router-chain-sdk-ts";
import { exec_msg } from "./execute_msg";
import { create_pair } from "./create_pair";
dotenv.config();

type GiveAllowance = {
    spender: string;
    amount: string;
};

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


    const dexSetupFilePath = "config/dex.json";
    const texchangeSetupFilePath = "config/texchange.json";
    const dexSetup = JSON.parse(fs.readFileSync(dexSetupFilePath, "utf-8"));
    const texchangeSetup = JSON.parse(fs.readFileSync(texchangeSetupFilePath, "utf-8"));
    console.log(dexSetup)
    console.log(texchangeSetup)

    const routerSwapFactoryAddr = dexSetup[network]["routerSwapFactory"]["addr"];
    if (!routerSwapFactoryAddr) {
        throw new Error("Not able to find 'routerSwapFactoryAddr' in dex Setup file");
    }

    const routerSwapRouterAddr = dexSetup[network]["routerSwapRouter"]["addr"];
    if (!routerSwapRouterAddr) {
        throw new Error("Not able to find 'routerSwapRouterAddr' in dex Setup file");
    }

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

    type SetConfig = {
        address: string;
    };
    const setRouteConfig: SetConfig = {
        address: routerSwapRouterAddr,
    };
    
    await exec_msg(texchangeAddr, "set_router_config", setRouteConfig);
    // for (let i = 0; i < logs[0].events.length; i++) {
    //     console.log(logs[0].events[i]);
    // }
    // console.log(logs);


    let pairInfo = await create_pair(routerSwapFactoryAddr, wrappedMaticAddr, wrappedUsdcAddr, "mt", "usd");
    let pair_contract_addr: string = pairInfo[0];
    let liquidity_token_addr: string = pairInfo[1];
    texchangeSetup[network]["maticusdcPair"] = {
        "addr": pair_contract_addr,
        "code_id": ""
    };
    texchangeSetup[network]["maticusdclp"] = {
        "addr": liquidity_token_addr,
        "code_id": ""
    };
    
    let giveAllowance: GiveAllowance = {
        spender: pair_contract_addr,
        amount: "1000000000000000000"
    }
    await exec_msg(wrappedMaticAddr, "increase_allowance", giveAllowance);
    await exec_msg(wrappedUsdcAddr, "increase_allowance", giveAllowance);

    pairInfo = await create_pair(routerSwapFactoryAddr, wrappedAvaxAddr, wrappedUsdcAddr, "avax", "usd");
    pair_contract_addr = pairInfo[0];
    liquidity_token_addr = pairInfo[1];
    texchangeSetup[network]["avaxusdcPair"] = {
        "addr": pair_contract_addr,
        "code_id": ""
    };
    texchangeSetup[network]["avaxusdclp"] = {
        "addr": liquidity_token_addr,
        "code_id": ""
    };
    
    fs.writeFileSync(texchangeSetupFilePath, JSON.stringify(texchangeSetup));

    giveAllowance = {
        spender: pair_contract_addr,
        amount: "1000000000000000000"
    }
    await exec_msg(wrappedAvaxAddr, "increase_allowance", giveAllowance);
    await exec_msg(wrappedUsdcAddr, "increase_allowance", giveAllowance);

}

main()
