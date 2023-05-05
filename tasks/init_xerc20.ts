import fs from "fs";
import dotenv from "dotenv";
import { init_wasm_code } from "./init_contract";
import { upload_wasm_code } from "./upload_wasm";
import { Network, PrivateKey } from "@routerprotocol/router-chain-sdk-ts";
import { exec_msg } from "./execute_msg";
import { query_balance } from "./perform_query";
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

  let balance = await query_balance(network, alice);
  if (balance === 0) {
    throw new Error(`Please maintain atleast 1 route balance in your account ${alice}`,);
  }
  const xerc20SetupFilePath = "./deployment/config/xerc20.json";
  const xerc20Setup = JSON.parse(fs.readFileSync(xerc20SetupFilePath, "utf-8"));
  console.log(xerc20Setup)

  // const cw20_token_code_id = await upload_wasm_code(
  //   network,
  //   privateKeyHash,
  //   chainId,
  //   "./cosmwasm/artifacts/cw20_token-aarch64.wasm"
  // );

  // const xerc20CodeId = await upload_wasm_code(
  //   network,
  //   privateKeyHash,
  //   chainId,
  //   "./cosmwasm/artifacts/xerc20-aarch64.wasm"
  // );

  // console.log("cw20_token code id -> ", cw20_token_code_id);
  // console.log("xerc20CodeId code id ->", xerc20CodeId);

  // const xerc20InitMsg = JSON.stringify({
  //   "cw20_code_id": parseInt(cw20_token_code_id),
  //   "token_name": "XERC20",
  //   "token_symbol": "XCW"
  // });
  // const xerc20Addr = await init_wasm_code(xerc20CodeId, "xerc20", xerc20InitMsg);
  // console.log("xerc20Addr", xerc20Addr);

  // let set_chain_types_info = {
  //   "chain_type_info": [
  //     {
  //       "chain_id": "80001",
  //       "chain_type": 1
  //     },
  //     {
  //       "chain_id": "43113",
  //       "chain_type": 1
  //     },
  //     {
  //       "chain_id": "5",
  //       "chain_type": 1
  //     },
  //     {
  //       "chain_id": "97",
  //       "chain_type": 1
  //     },
  //     {
  //       "chain_id": "router_9000-1",
  //       "chain_type": 2
  //     }
  //   ]
  // };

  // await exec_msg(xerc20Addr, "set_chain_types", set_chain_types_info);
  // console.log("admin ->", alice);
  // console.log("xerc20 -> code_id-", xerc20CodeId, "addr-", xerc20Addr);

  let set_white_listed_contracts = {
    contracts: [
      {
        chain_id: "80001",
        contract_addr: "0x3db0994d4591727117f28a29f56dcff09f0ec63b"
      }
    ]
  };

  await exec_msg(xerc20Setup[network]["xerc20"]["addr"], "set_white_listed_contracts", set_white_listed_contracts);
  // 
  // if (!xerc20Setup[network]) {
  //   xerc20Setup[network] = {};
  // }
  // xerc20Setup[network]["xerc20"] = {
  //   "addr": xerc20Addr,
  //   "code_id": xerc20CodeId
  // };
  // xerc20Setup[network]["cw20_token"] = {
  //   "addr": "",
  //   "code_id": cw20_token_code_id
  // };

  // fs.writeFileSync(xerc20SetupFilePath, JSON.stringify(xerc20Setup));
}

main()
