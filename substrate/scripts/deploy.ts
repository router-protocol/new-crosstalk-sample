import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { Abi } from "@polkadot/api-contract";
import fs from "fs";
import RouteToken_factory from "../types/constructors/route_token";
import Gateway_factory from "../types/constructors/gateway_contract";
import Dapp_factory from "../types/constructors/dapp";
import RouteToken from "../types/contracts/route_token";
import {
  DeploymentStore,
  parseCommandLineArgs,
  ss558AccountIdFromHexEncodedString,
  ss558AccountIdToHexEncodedString,
} from "./utils";
import { getNetwork } from "./config/chain.config";
import args from "./config/args.json";
import { KeyringPair } from "@polkadot/keyring/types";

require("dotenv").config();

// ts-node ./scripts/deploy.ts --net "aleph-testnet" --type "route_token"
// ts-node ./scripts/deploy.ts --net "aleph-testnet" --type "dapp" --gateway 5CcDvdYaiSzSJqboXjeAj79W7AjfefrWkGEcWinPpHprkHMY
// ts-node ./scripts/deploy.ts --net "aleph-testnet" --type "gateway" --routetoken 5ERU7SyfzQ8xYyPYYZfqZvaUEUksyALTusaDXBpaXhTHtMUE
const MINTER_ROLE = 4254773782;
async function main() {
  const store = new DeploymentStore();
  let { net, env, routetoken, type, gateway } = parseCommandLineArgs();
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
    case "dapp":
      await DappDeployment(deployer, api, store, env, network, gateway);
      break;
    case "gateway":
      await GatewayDeployment(deployer, api, store, env, network, routetoken);
      break;
    case "route_token":
      await RouteTokenDeployment(deployer, api, store, env, network);
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

async function DappDeployment(
  deployer: KeyringPair,
  api: ApiPromise,
  store: DeploymentStore,
  env: string,
  network: any,
  gateway: string
) {
  console.log("|| Deploying Dapp ||");
  const dappFactory = new Dapp_factory(api, deployer);
  const dappRaw = JSON.parse(
    fs.readFileSync(__dirname + `/../artifacts/dapp.contract`, "utf8")
  );
  const dappRawABI = new Abi(dappRaw);
  gateway = gateway
    ? gateway
    : (await store.getStore())[env][network.id]["Gateway"].gateway.address;
  //@ts-ignore
  let { gasRequired } = await api.call.contractsApi.instantiate(
    deployer.address,
    0,
    null,
    null,
    { Upload: dappRawABI.info.source.wasm },
    dappRawABI.constructors[0].toU8a([network.id, gateway]),
    ""
  );
  const { address, result } = await dappFactory.new(network.id, gateway, {
    gasLimit: gasRequired,
  });
  console.log(
    "[ Dapp Deployed To | SS558 Address: ",
    address,
    ", HexAddress: ",
    ss558AccountIdToHexEncodedString(api, address),
    " ]"
  );
  const { block } = await api.rpc.chain.getBlock(result.blockHash);
  await store.store(env, network.id, "Dapp", "", "", {
    dapp: {
      address: address,
      txHash: result.txHash,
      blockHash: result.blockHash,
      blockNumber: block.header.number.toString(),
    },
    creationTime: Date.now(),
  });
}
