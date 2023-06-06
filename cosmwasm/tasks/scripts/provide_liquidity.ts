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
    

    const avaxusdcPairAddr = texchangeSetup[network]["avaxusdcPair"]["addr"];
    if (!avaxusdcPairAddr) {
        throw new Error("Not able to find 'avaxusdcPairAddr' in dex Setup file");
    }

    const maticusdcPairAddr = texchangeSetup[network]["maticusdcPair"]["addr"];
    if (!maticusdcPairAddr) {
        throw new Error("Not able to find 'maticusdcPairAddr' in texchange Setup file");
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

    let provideLiquidity = {
        "assets":[
            {
                "info": {
                    "token": {
                        "contract_addr": wrappedMaticAddr
                    }
                },
                "amount": "1000000000000000000"
            },
            {
                "info": {
                    "token": {
                        "contract_addr": wrappedUsdcAddr
                    }
                },
                "amount": "500000000000000000"
            }
        ]
    };
    console.log(maticusdcPairAddr, JSON.stringify(provideLiquidity));
    let logs = await exec_msg(maticusdcPairAddr, "provide_liquidity", provideLiquidity);
    console.log(logs);

    provideLiquidity = {
        "assets":[
            {
                "info": {
                    "token": {
                        "contract_addr": wrappedAvaxAddr
                    }
                },
                "amount": "1000000000000000000"
            },
            {
                "info": {
                    "token": {
                        "contract_addr": wrappedUsdcAddr
                    }
                },
                "amount": "500000000000000000"
            }
        ]
    };
    console.log(JSON.stringify(provideLiquidity));
    logs = await exec_msg(avaxusdcPairAddr, "provide_liquidity", provideLiquidity);
    console.log(logs);
}

main()
