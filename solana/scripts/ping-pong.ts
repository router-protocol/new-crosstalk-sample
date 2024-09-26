import dotenv from "dotenv";
import { getClustorConfig, parseCommandLineArgs } from "./utils";
import { ping, initialize, setDappMetadata } from "./ping-pong.utils";
import { Keypair, PublicKey } from "@solana/web3.js";
import { ethers } from "ethers";
import * as anchor from "@coral-xyz/anchor";
import { PingPong } from "../target/types/ping_pong";

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
    const privateKey = process.env.PRIVATE_KEY;
    if (!privateKey) throw new Error("provide PRIVATE_KEY in .env");
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
    const provider = anchor.AnchorProvider.local(clustorConfig.url);

    const ping_pong = anchor.workspace.PingPong as anchor.Program<PingPong>;
    if (program_id) program_id = ping_pong.programId;
    const pingPongInstance = new anchor.Program(
      ping_pong.idl,
      new PublicKey(program_id),
      provider
    );
    switch (type) {
      //NOTE: ts-node ./scripts/ping-pong.ts --type "initialize" --net solana-devnet --program_id BaLkN7XPeRjrrh4fAaNDsZhX5aade46ZkDCRTKTviyHV --args "solana-devnet,ABcpcW3w3mStkUHh1tsftefxksajtezA9C1r6t59zf5A"
      case "initialize":
        await initialize(provider, pingPongInstance, deployer, args.split(","));
        break;
      //NOTE: ts-node ./scripts/ping-pong.ts --type "set_dapp_metadata" --net solana-devnet --program_id BaLkN7XPeRjrrh4fAaNDsZhX5aade46ZkDCRTKTviyHV  --args "Aak2MJfJAhFk3vmg2LG97hmNa3TUtKzn4kM7FgWYLw5F,0x4E27128CdEF7a3CFFdF800BE3Be6EE74639CB639"
      case "set_dapp_metadata":
        await setDappMetadata(
          provider,
          pingPongInstance,
          deployer,
          args.split(",")
        );
        break;
      //NOTE: ts-node ./scripts/ping-pong.ts --type "ping" --net solana-devnet --program_id BaLkN7XPeRjrrh4fAaNDsZhX5aade46ZkDCRTKTviyHV --args "Aak2MJfJAhFk3vmg2LG97hmNa3TUtKzn4kM7FgWYLw5F,43113,0xC44ce1FE770EdC617d4917159a65386885Dc6619"
      case "ping":
        await ping(provider, pingPongInstance, deployer, args.split(","));
        break;
    }
  } catch (e) {
    console.log(e);
  }
  process.exit(1);
}
ifn().catch(console.log);
