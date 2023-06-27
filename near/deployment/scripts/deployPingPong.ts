import fs from "fs-extra";
import dotenv from "dotenv";
import * as nearAPI from "near-api-js";
import BN from "bn.js";
import { approve } from "./approveFeePayer";
import { getGateway } from "./utils";
dotenv.config();

export async function deployPingPong() {
  try {
    const routerNetwork = process.env.ROUTER_NETWORK;
    if (!routerNetwork) {
      throw new Error("Please add ROUTER_NETWORK to .env file");
    }

    let nearNetwork = "testnet";
    if (routerNetwork == "mainnet") {
      nearNetwork = "mainnet";
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

    const timestamp = Date.now();
    const pingPongKeyPair = nearAPI.KeyPair.fromRandom("ed25519");
    const pingPongAddress = "ping-pong-" + timestamp + "." + signerAddress;

    await account.createAccount(
      pingPongAddress, // new account name
      pingPongKeyPair.getPublicKey(), // public key for new account
      new BN("20000000000000000000000000") // initial balance for new account in yoctoNEAR
    );
    const pingPongAccount = await nearConnection.account(pingPongAddress);

    await myKeyStore.setKey(nearNetwork, pingPongAddress, pingPongKeyPair);

    console.log("Deploying Ping Pong started");

    const C11 = await pingPongAccount.deployContract(
      fs.readFileSync("../artifacts/ping_pong.wasm")
    );

    console.log("Ping Pong deployment status: ", C11.status);
    console.log("Deployed Ping Pong to ", pingPongAddress);

    console.log("Deploying Gateway started");

    console.log("Storing data started");
    const data = await fs.readJSONSync("deployment.json");

    if (!data[routerNetwork]) {
      data[routerNetwork] = {};
    }

    data[routerNetwork].pingPong = pingPongAddress;

    fs.writeJSONSync("deployment.json", data);
    console.log("Storing data ended");

    const pingPongContract: any = new nearAPI.Contract(
      account,
      pingPongAddress,
      {
        changeMethods: ["new", "set_dapp_metadata"], // your smart-contract has a function `my_smart_contract_function`
        viewMethods: [],
      }
    );

    let gateway = await getGateway();
    if (gateway == "") {
      gateway = data[routerNetwork].gateway;
    }

    console.log("Initializing PingPong contract started");
    await pingPongContract.new({
      args: {
        gateway,
      },
      gas: 50000000000000,
    });
    console.log("Initializing PingPong contract contract ended");

    console.log("Setting fee payer address started");
    await pingPongContract.set_dapp_metadata({
      args: {
        fee_payer_address: data[routerNetwork].feePayer,
      },
      gas: 50000000000000,
    });
    console.log("Setting fee payer address ended");

    console.log("Waiting for a minute started for approving fee payer");
    await new Promise((r) => setTimeout(r, 60000));
    console.log("Waiting for a minute ended for approving fee payer");

    console.log("Providing fee payer approval started");
    await approve(data[routerNetwork].chainId, pingPongAddress);
    console.log("Providing fee payer approval ended");
  } catch (e) {
    console.log(e);
  }
}

deployPingPong();
