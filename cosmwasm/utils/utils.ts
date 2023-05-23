import {
  ChainRestAuthApi,
  Network,
  PrivateKey,
  TxGrpcClient,
  TxRestClient,
  getEndpointsForNetwork,
} from "@routerprotocol/router-chain-sdk-ts";
import dotenv from "dotenv";
import path from "path";
import fs from "fs-extra";

dotenv.config();

type UserPrivateInfo = {
  mnemonic?: string;
  privateKey?: string;
  isMnemonic?: boolean;
};

export async function getAccount(
  privateInfo: UserPrivateInfo,
  network: Network
) {
  const endpoint = getEndpointsForNetwork(network);

  let privateKey: PrivateKey;
  if (privateInfo.isMnemonic) {
    if (!privateInfo.mnemonic) {
      console.log("Provide Mnemonic, if isMnemonic is true");
      process.exit(1);
    }
    privateKey = PrivateKey.fromMnemonic(privateInfo.mnemonic);
  } else {
    if (!privateInfo.privateKey) {
      console.log("Provide private key, if isMnemonic is false");
      process.exit(1);
    }
    privateKey = PrivateKey.fromMnemonic(privateInfo.privateKey);
  }

  const alice = privateKey.toBech32();
  const publicKey = privateKey.toPublicKey().toBase64();

  /** Get Faucet Accounts details */
  const aliceAccount = await new ChainRestAuthApi(
    endpoint.lcdEndpoint
  ).fetchAccount(alice);
  return aliceAccount;
}

export function getNetworkFromEnv(): Network {
  let network = Network.AlphaDevnet;
  if (process.env.NETWORK == "devnet") {
    network = Network.Devnet;
  } else if (process.env.NETWORK == "testnet") {
    network = Network.Testnet;
  } else if (process.env.NETWORK == "mainnet") {
    network = Network.Mainnet;
  } else if (process.env.NETWORK && process.env.NETWORK != "alpha-devnet") {
    throw new Error("Please set your NETWORK in the .env file");
  }
  return network;
}

const deploymentPath = path.join(__dirname, "../deployment");

export async function getPrevDeployment() {
  await fs.ensureDir(deploymentPath);
  const deployment = await fs
    .readJSON(`${deploymentPath}/deployment.json`)
    .catch(() => ({}));
  return deployment;
}

export async function updatePrevDeployment(Obj: Object) {
  await fs.writeJSON(`${deploymentPath}/deployment.json`, Obj);
}
