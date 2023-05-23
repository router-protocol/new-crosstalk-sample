import fs from "fs-extra";
import dotenv from "dotenv";
import { init_wasm_code } from "../utils/init_wasm";
import { upload_wasm_code } from "../utils/upload_wasm";
import {
  Network,
  getChainInfoForNetwork,
} from "@routerprotocol/router-chain-sdk-ts";
import path from "path";
import { getAccount, getNetworkFromEnv } from "../utils/utils";

dotenv.config();

const deploymentPath = path.join(__dirname, "../deployment");

async function main() {
  let network = getNetworkFromEnv();

  const chainInfo = getChainInfoForNetwork(network);
  const chainId = chainInfo.chainId;

  let wasmSuffix = ".wasm";
  if (process.env.IS_APPLE_CHIPSET == "YES") {
    wasmSuffix = "-aarch64.wasm";
  }

  const userInfo = {
    isMnemonic: true,
    mnemonic: process.env.MNEMONIC,
  };
  console.log("Uploading RouterRider Contract, Please wait...");
  const routerRiderCodeId = await upload_wasm_code(
    network,
    chainId,
    __dirname + "/../artifacts/router_rider" + wasmSuffix,
    {
      isMnemonic: true,
      mnemonic: process.env.MNEMONIC,
    }
  );
  console.log("RouterRider CodeId: ", routerRiderCodeId);

  console.log("Uploading Lottery Contract, Please wait...");
  const lotteryCodeId = await upload_wasm_code(
    network,
    chainId,
    __dirname + "/../artifacts/lottery" + wasmSuffix,
    {
      isMnemonic: true,
      mnemonic: process.env.MNEMONIC,
    }
  );
  console.log("Lottery CodeId: ", lotteryCodeId);

  console.log("Initiating RouterRider Contract, Please wait...");
  const routerRiderInitMsg = JSON.stringify({
    chain_id: chainId,
    dest_gas_limit: 1000000,
    ack_gas_limit: 1000000,
    cooldown_time: "5",
    relayer_fee: "10",
  });
  const routerRiderAddress = await init_wasm_code(
    network,
    routerRiderCodeId,
    "Wrapped Route",
    routerRiderInitMsg,
    chainId,
    userInfo
  );
  console.log("RouterRider Address: ", routerRiderAddress);

  console.log("Initiating Lottery Contract, Please wait...");
  const deployerAccount = await getAccount(
    {
      isMnemonic: true,
      mnemonic: process.env.MNEMONIC,
    },
    network
  );
  const lotteryInitMsg = JSON.stringify({
    owner: deployerAccount.account.base_account.address,
    _pd: 1800, //30min
    _ld: 3600, // 1hour
  });
  const lotteryAddress = await init_wasm_code(
    network,
    lotteryCodeId,
    "Wrapped Route",
    lotteryInitMsg,
    chainId,
    userInfo
  );
  console.log("Lottery Address: ", lotteryAddress);

  await fs.ensureDir(deploymentPath);
  await fs.writeJSON(`${deploymentPath}/deployment.json`, {
    routerRider: {
      codeId: routerRiderCodeId,
      address: routerRiderAddress,
    },
    lottery: {
      codeId: lotteryCodeId,
      address: lotteryAddress,
    },
  });
}

main();
