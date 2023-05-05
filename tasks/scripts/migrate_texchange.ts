import fs from "fs";
import dotenv from "dotenv";
import { Network, PrivateKey } from "@routerprotocol/router-chain-sdk-ts";
import { upload_wasm_code } from "./upload_wasm";
import { migrateContract } from "./migrate";

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
    const texchangeSetup = JSON.parse(
        fs.readFileSync(texchangeSetupFilePath, "utf-8")
    );

    const texchangeAddr = texchangeSetup[network]["texchange"]["addr"];
    if (!texchangeAddr) {
        throw new Error("Not able to find 'texchangeAddr' in texchange Setup file");
    }

    const texchangeCodeId = await upload_wasm_code(
        network,
        privateKeyHash,
        chainId,
        "../middleware/artifacts/texchange_bridge.wasm"
      );

    let jsonMsg = "{}";
    let msgObject = JSON.parse(jsonMsg);
    await migrateContract(texchangeAddr, parseInt(texchangeCodeId), msgObject);
    console.log("Setting Resources Complete");

}

main();
