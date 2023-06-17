import fs from "fs-extra";
import dotenv from "dotenv";
import * as nearAPI from "near-api-js";
dotenv.config();

const requestMetadata: { [key: string]: Array<number> } = {
  EVM: [
    0, 0, 0, 0, 0, 15, 66, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 121, 136, 61,
    32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0,
  ],
  ROUTER: [
    0, 0, 0, 0, 0, 45, 198, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45, 121, 136, 61,
    32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0,
  ],
};

export async function sendPing(
  destChainId: string,
  str: string,
  recipient: string
) {
  console.log("Sending Ping started");

  const routerNetwork = process.env.ROUTER_NETWORK;
  if (!routerNetwork) {
    throw new Error("Please add ROUTER_NETWORK to .env file");
  }

  const data = await fs.readJSONSync("scripts/deployment.json");
  const pingPongAddr = data[routerNetwork].PingPong;

  const destFile = await fs.readJSONSync("scripts/chains.json");
  const destContractAddress = destFile[routerNetwork][destChainId].PingPong;
  const destChainType = destFile[routerNetwork][destChainId].chainType;

  const nearNetwork = process.env.NEAR_NETWORK;
  if (!nearNetwork) {
    throw new Error("Please add NEAR_NETWORK to .env file");
  }

  const signerAddress = process.env.NEAR_SIGNER_ADDRESS;
  if (!signerAddress) {
    throw new Error("Please add NEAR_SIGNER_ADDRESS to .env file");
  }

  const { keyStores, KeyPair, connect } = nearAPI;

  const homedir = require("os").homedir();
  const CREDENTIALS_DIR = ".near-credentials";
  const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);

  const myKeyStore = new keyStores.UnencryptedFileSystemKeyStore(
    credentialsPath
  );

  if ((await myKeyStore.getKey(nearNetwork, signerAddress)) == null) {
    const privateKey = process.env.NEAR_PRIVATE_KEY;
    if (!privateKey) {
      throw new Error("Please add NEAR_PRIVATE_KEY to .env file");
    }

    // creates a public / private key pair using the provided private key
    const keyPair = KeyPair.fromString(privateKey);

    // adds the keyPair you created to keyStore
    await myKeyStore.setKey(nearNetwork, signerAddress, keyPair);
  }

  const connectionConfig = {
    networkId: nearNetwork,
    keyStore: myKeyStore, // first create a key store
    nodeUrl: `https://rpc.${nearNetwork}.near.org`,
    walletUrl: `https://wallet.${nearNetwork}.near.org`,
    helperUrl: `https://helper.${nearNetwork}.near.org`,
    explorerUrl: `https://explorer.${nearNetwork}.near.org`,
  };
  const nearConnection = await connect(connectionConfig);

  const account = await nearConnection.account(signerAddress);

  const pingPongInstance: any = new nearAPI.Contract(account, pingPongAddr, {
    changeMethods: ["i_ping"],
    viewMethods: [],
  });

  const reqMetadata = requestMetadata[destChainId];

  const tx = await pingPongInstance.i_ping({
    args: {
      dest_chain_id: destChainId,
      destination_contract_address: destContractAddress,
      str: str,
      request_metadata: requestMetadata[destChainType],
      recipient: recipient,
    },
    gas: 100000000000000,
    amount: 1,
  });

  console.log(tx);
}
