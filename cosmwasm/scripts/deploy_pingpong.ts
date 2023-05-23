import dotenv from "dotenv";
import { init_wasm_code } from "../utils/init_wasm";
import { upload_wasm_code } from "../utils/upload_wasm";
import { getChainInfoForNetwork } from "@routerprotocol/router-chain-sdk-ts";
import {
  getAccount,
  getNetworkFromEnv,
  getPrevDeployment,
  updatePrevDeployment,
} from "../utils/utils";
dotenv.config();

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
  console.log("Uploading PingPong Contract, Please wait...");
  const pingPongCodeId = await upload_wasm_code(
    network,
    chainId,
    __dirname + "/../artifacts/ping_pong" + wasmSuffix,
    userInfo
  );
  console.log("PingPong CodeId: ", pingPongCodeId);

  console.log("Initiating PingPong Contract, Please wait...");
  const pingPongInitMsg = JSON.stringify({});
  const pingPongAddress = await init_wasm_code(
    network,
    pingPongCodeId,
    "Wrapped Route",
    pingPongInitMsg,
    chainId,
    userInfo
  );
  console.log("PingPong Address: ", pingPongAddress);

  console.log("Updating deployment...");
  const prevDeployment = await getPrevDeployment();
  await updatePrevDeployment({
    ...prevDeployment,
    ping_pong: {
      codeId: pingPongCodeId,
      address: pingPongAddress,
    },
  });
  console.log("Updated deployment!!");
}

main();
