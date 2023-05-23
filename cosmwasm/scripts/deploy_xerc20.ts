import dotenv from "dotenv";
import { init_wasm_code } from "../utils/init_wasm";
import { upload_wasm_code } from "../utils/upload_wasm";
import { getChainInfoForNetwork } from "@routerprotocol/router-chain-sdk-ts";
import {
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

  console.log("Uploading cw20 Token Contract, Please wait...");
  const cw20CodeId =
    "149" ||
    (await upload_wasm_code(
      network,
      chainId,
      __dirname + "/../artifacts/cw20_token" + wasmSuffix,
      userInfo
    ));
  console.log("cw20 Token CodeId: ", cw20CodeId);

  console.log("Uploading XERC20 Token Contract, Please wait...");
  const xerc20CodeId =
    "150" ||
    (await upload_wasm_code(
      network,
      chainId,
      __dirname + "/../artifacts/xerc20" + wasmSuffix,
      userInfo
    ));
  console.log("XERC20 CodeId: ", xerc20CodeId);

  console.log("Initiating XERC20 Contract, Please wait...");
  const xerc20InitMsg = JSON.stringify({
    cw20_code_id: cw20CodeId,
    token_name: "XERC20",
    token_symbol: "XERC20",
  });
  const xerc20Address = await init_wasm_code(
    network,
    xerc20CodeId,
    "Wrapped Route",
    xerc20InitMsg,
    chainId,
    userInfo
  );
  console.log("xerc20 Address: ", xerc20Address);

  console.log("Updating deployment...");
  const prevDeployment = await getPrevDeployment();
  await updatePrevDeployment({
    ...prevDeployment,
    cw20: {
      codeId: cw20CodeId,
      address: "",
    },
    xerc20: {
      codeId: xerc20CodeId,
      address: xerc20Address,
    },
  });
  console.log("Updated deployment!!");
}

main();
