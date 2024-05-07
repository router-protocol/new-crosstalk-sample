import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { Abi } from "@polkadot/api-contract";
import fs from "fs";
import TestDapp_Factory from "../types/constructors/test_dapp";
import {
  DeploymentStore,
  parseCommandLineArgs,
  ss558AccountIdFromHexEncodedString,
  ss558AccountIdToHexEncodedString,
} from "./utils";
import { getNetwork } from "./config/chain.config";
import { KeyringPair } from "@polkadot/keyring/types";

require("dotenv").config();

// ts-node ./scripts/deploy.ts --net "aleph-testnet" --type "test-dapp" --gateway 5CcDvdYaiSzSJqboXjeAj79W7AjfefrWkGEcWinPpHprkHMY --routetoken 5ERU7SyfzQ8xYyPYYZfqZvaUEUksyALTusaDXBpaXhTHtMUE --feepayer 0x4A7239751857bCc1Cd75485FdE66B07224C448Bf
async function main() {
  const store = new DeploymentStore();
  let { net, env, routetoken, feepayer, type, gateway } =
    parseCommandLineArgs();
  if (!net) throw new Error("network undefined, pass as --net");
  if (!env) env = "testnet";
  if (!type) type = "gateway";

  const network = getNetwork(net);
  if (!network) throw new Error("Invalid Network!!");

  const keyring = new Keyring({ type: network.type });
  const wsProvider = new WsProvider(network.rpcs[0]);
  const api = await ApiPromise.create({ provider: wsProvider });

  //   const deployer = keyring.addFromUri(process.env.PRIVATE_KEY);
  const deployer = keyring.addFromMnemonic(process.env.MNEMONIC);

  switch (type) {
    case "test-dapp":
      await TesDappDeployment(
        deployer,
        api,
        store,
        env,
        network,
        gateway,
        feepayer,
        routetoken
      );
      break;
    default:
      throw new Error("Invalid Deployment Type!!");
  }
  // Gateway Deployment
}

main()
  .then(() => {
    process.exit(1);
  })
  .catch((err) => {
    console.log(err);
    process.exit(0);
  });

async function TesDappDeployment(
  deployer: KeyringPair,
  api: ApiPromise,
  store: DeploymentStore,
  env: string,
  network: any,
  gateway: string,
  feepayer: string,
  routetoken: string
) {
  console.log("|| Deploying Test Dapp ||");
  const testDappFactory = new TestDapp_Factory(api, deployer);
  const testDappRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/test_dapp.contract`, "utf8")
  );
  const dappRawABI = new Abi(testDappRaw);
  gateway = gateway
    ? gateway
    : (await store.getStore())[env][network.id]["TestDapp"].gateway.address;
  //@ts-ignore
  let { gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: dappRawABI.info.source.wasm },
    dappRawABI.constructors[0].toU8a([gateway, feepayer, routetoken]),
    ""
  );
  const { address, result } = await testDappFactory.new(
    gateway,
    feepayer,
    routetoken,
    {
      gasLimit: gasRequired,
    }
  );
  console.log(
    "[ Test Dapp Deployed To | SS558 Address: ",
    address,
    ", HexAddress: ",
    ss558AccountIdToHexEncodedString(api, address),
    " ]"
  );
  const { block } = await api.rpc.chain.getBlock(result.blockHash);
  await store.store(env, network.id, "TestDapp", "", "", {
    dapp: {
      address: address,
      txHash: result.txHash,
      blockHash: result.blockHash,
      blockNumber: block.header.number.toString(),
    },
    creationTime: Date.now(),
  });
}
