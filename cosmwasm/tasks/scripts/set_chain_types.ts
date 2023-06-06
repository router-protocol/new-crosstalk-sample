import fs from "fs";
import dotenv from "dotenv";
import { exec_msg } from "./execute_msg";
import { Network } from "@routerprotocol/router-chain-sdk-ts";

dotenv.config();

async function main() {
    let network = Network.AlphaDevnet;
    if (process.env.NETWORK == "devnet") {
        network = Network.Devnet;
    } else if (process.env.NETWORK == "testnet") {
        network = Network.Testnet;
    } else if (process.env.NETWORK == "mainnet") {
        network = Network.Mainnet;
    } else if (process.env.NETWORK && process.env.NETWORK != "alpha-devnet") {
        throw new Error("Please set your NETWORK in the .env file");
    }

    const texchangeSetupFilePath = "config/texchange.json";
    const texchangeSetup = JSON.parse(
        fs.readFileSync(texchangeSetupFilePath, "utf-8")
    );

    const texchangeAddr = texchangeSetup[network]["texchange"]["addr"];
    if (!texchangeAddr) {
        throw new Error("Not able to find 'texchangeAddr' in texchange Setup file");
    }

    let set_chain_types_info = {
        "chain_type_info": [
            {
                "chain_id": "80001",
                "chain_type": 1
            },
            {
                "chain_id": "43113",
                "chain_type": 1
            },
            {
                "chain_id": "5",
                "chain_type": 1
            },
            {
                "chain_id": "97",
                "chain_type": 1
            },
            {
                "chain_id": "router_9000-1",
                "chain_type": 2
            }
        ]
    };

    await exec_msg(texchangeAddr, "set_chain_types", set_chain_types_info);
    console.log("Setting Resources Complete");

}

main();
