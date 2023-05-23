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

  const { account } = await getAccount(userInfo, network);

  console.log("Uploading XERC721 Contract, Please wait...");
  const xerc721CodeId = await upload_wasm_code(
    network,
    chainId,
    __dirname + "/../artifacts/XERC721" + wasmSuffix,
    userInfo
  );
  console.log("XERC721 CodeId: ", xerc721CodeId);

  console.log("Initiating XERC721 Contract, Please wait...");
  const xerc721InitMsg = JSON.stringify({
    name: "XERC721",
    symbol: "XERC721",
    //@ts-ignore
    minter: account.base_account.address,
  });
  const xerc721Address = await init_wasm_code(
    network,
    xerc721CodeId,
    "Wrapped Route",
    xerc721InitMsg,
    chainId,
    userInfo
  );
  console.log("XERC721 Address: ", xerc721Address);

  console.log("Updating deployment...");
  const prevDeployment = await getPrevDeployment();
  await updatePrevDeployment({
    ...prevDeployment,
    xerc721: {
      codeId: xerc721CodeId,
      address: xerc721Address,
    },
  });
  console.log("Updated deployment!!");
}

main();
