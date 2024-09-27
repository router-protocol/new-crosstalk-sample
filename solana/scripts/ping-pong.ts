import dotenv from "dotenv";
import { getClustorConfig, parseCommandLineArgs } from "./utils";
import {
  ping,
  initialize,
  setDappMetadata,
  getDstContract,
  approveFeePayer,
} from "./ping-pong.utils";
import { Keypair, PublicKey } from "@solana/web3.js";
import fse from "fs-extra";

import { ethers } from "ethers";
import * as anchor from "@coral-xyz/anchor";
import { IDL } from "../target/types/ping_pong";
import os from "os";
import path from "path";

dotenv.config();

export async function ifn() {
  try {
    let { net, env, type, program_id, args } = parseCommandLineArgs();
    if (!net) throw new Error("network undefined, pass as --net");
    if (!env) env = "solana-testnet";
    if (!type)
      throw new Error(
        "Please provide type --type ...type, can be `irelay, ireceive, isend`"
      );
    let privateKey = process.env.PRIVATE_KEY;
    if (!privateKey)
      try {
        privateKey = ethers.hexlify(
          Uint8Array.from(
            await fse.readJson(
              path.join(os.homedir(), ".config/solana/id.json")
            )
          )
        );
      } catch (error) {
        throw new Error(`provide PRIVATE_KEY in .env] Error: ${error}`);
      }

    const deployer = privateKey.startsWith("[")
      ? Keypair.fromSecretKey(
          Uint8Array.from(JSON.parse(privateKey.replace(/\s+/g, "")))
        )
      : Keypair.fromSecretKey(
          ethers.getBytes(
            privateKey.startsWith("0x") ? privateKey : `0x${privateKey}`
          )
        );
    const clustorConfig = getClustorConfig(net);
    const provider = new anchor.AnchorProvider(
      new anchor.web3.Connection(clustorConfig.url),
      new anchor.Wallet(deployer),
      { "commitment": "confirmed" }
    );
    if (!program_id) throw new Error(`provide ping-pong address`);
    const pingPongInstance = new anchor.Program(
      IDL,
      new PublicKey(program_id),
      provider
    );
    switch (type) {
      //NOTE: ts-node ./scripts/ping-pong.ts --type "initialize" --net solana-devnet --program_id 7dQqaHQFRBC8AhaRzqtQWLEM7fXxwGw9VEKwLpyf8rM3 --args "solana-devnet,Aak2MJfJAhFk3vmg2LG97hmNa3TUtKzn4kM7FgWYLw5F"
      case "initialize":
        await initialize(provider, pingPongInstance, deployer, args.split(","));
        break;
      //NOTE: ts-node ./scripts/ping-pong.ts --type "set_dapp_metadata" --net solana-devnet --program_id 7dQqaHQFRBC8AhaRzqtQWLEM7fXxwGw9VEKwLpyf8rM3  --args "Aak2MJfJAhFk3vmg2LG97hmNa3TUtKzn4kM7FgWYLw5F,0x4E27128CdEF7a3CFFdF800BE3Be6EE74639CB639"
      case "set_dapp_metadata":
        await setDappMetadata(
          provider,
          pingPongInstance,
          deployer,
          args.split(",")
        );
        break;
      //NOTE: ts-node ./scripts/ping-pong.ts --type "approve_fee_payer" --net solana-devnet --program_id 7dQqaHQFRBC8AhaRzqtQWLEM7fXxwGw9VEKwLpyf8rM3 --args "0x62aca6e7b2560126cd03fcd5b355aa385fe37135c2779cf185d822fa0c2d123d,testnet"
      case "approve_fee_payer":
        await approveFeePayer(
          provider,
          pingPongInstance,
          deployer,
          args.split(",")
        );
        break;
      //NOTE: ts-node ./scripts/ping-pong.ts --type "ping" --net solana-devnet --program_id 7dQqaHQFRBC8AhaRzqtQWLEM7fXxwGw9VEKwLpyf8rM3 --args "Aak2MJfJAhFk3vmg2LG97hmNa3TUtKzn4kM7FgWYLw5F,43113,0xC44ce1FE770EdC617d4917159a65386885Dc6619"
      case "ping":
        await ping(provider, pingPongInstance, deployer, args.split(","));
        break;
      //NOTE: ts-node ./scripts/ping-pong.ts --type "get_dst_contract" --net solana-devnet --program_id 7dQqaHQFRBC8AhaRzqtQWLEM7fXxwGw9VEKwLpyf8rM3
      case "get_dst_contract":
        await getDstContract(provider, pingPongInstance, deployer, []);
        break;
    }
  } catch (e) {
    console.log(e);
  }
  process.exit(1);
}
ifn().catch(console.log);
