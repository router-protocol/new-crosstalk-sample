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

  console.log("Uploading cw1155 Token Contract, Please wait...");
  const cw1155CodeId = await upload_wasm_code(
    network,
    chainId,
    __dirname + "/../artifacts/cw1155_token" + wasmSuffix,
    userInfo
  );
  console.log("cw1155 Token CodeId: ", cw1155CodeId);

  console.log("Uploading XERC1155 Token Contract, Please wait...");
  const xerc1155CodeId = await upload_wasm_code(
    network,
    chainId,
    __dirname + "/../artifacts/XERC1155" + wasmSuffix,
    userInfo
  );
  console.log("XERC1155 CodeId: ", xerc1155CodeId);

  console.log("Initiating XERC1155 Contract, Please wait...");
  const xerc1155InitMsg = JSON.stringify({
    cw1155_codeid: cw1155CodeId,
    minter: account.base_account.address,

    // token_name: "XERC1155",
    // token_symbol: "XERC1155",
  });
  const xerc1155Address = await init_wasm_code(
    network,
    xerc1155CodeId,
    "Wrapped Route",
    xerc1155InitMsg,
    chainId,
    userInfo
  );
  console.log("xerc1155 Address: ", xerc1155Address);

  console.log("Updating deployment...");
  const prevDeployment = await getPrevDeployment();
  await updatePrevDeployment({
    ...prevDeployment,
    cw1155: {
      codeId: cw1155CodeId,
      address: "",
    },
    xerc1155: {
      codeId: xerc1155CodeId,
      address: xerc1155Address,
    },
  });
  console.log("Updated deployment!!");
}

main();
