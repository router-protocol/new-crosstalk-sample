import {
  getEndpointsForNetwork,
  PrivateKey,
  privateKeyToPublicKeyBase64,
  ChainRestAuthApi,
  createTransaction,
  BigNumberInBase,
  TxRestClient,
  Network,
  MsgStoreCode,
  TxGrpcClient,
  TxClientSimulateResponse,
} from "@routerprotocol/router-chain-sdk-ts";
import fs from "fs";
import dotenv from "dotenv";
import { parseRawLog } from "@cosmjs/stargate/build/logs";
import { logs } from "@cosmjs/stargate";
dotenv.config();

export type UserPrivateInfo = {
  mnemonic?: string;
  privateKey?: string;
  isMnemonic?: boolean;
};

export const upload_wasm_code = async function (
  network: Network,
  chainId: string,
  wasmFilePath: string,
  privateInfo: UserPrivateInfo
): Promise<string> {
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

  const restClient = new TxRestClient(endpoint.lcdEndpoint);
  const grpcClient = new TxGrpcClient(endpoint.grpcEndpoint);
  /** Get Faucet Accounts details */
  const aliceAccount = await new ChainRestAuthApi(
    endpoint.lcdEndpoint
  ).fetchAccount(alice);

  const wasmFile = fs.readFileSync(wasmFilePath);
  const storeCodeMsg = MsgStoreCode.fromJSON({
    sender: alice,
    wasm: wasmFile,
  });
  let simulationResponse: TxClientSimulateResponse;
  {
    let { txRaw } = createTransaction({
      message: storeCodeMsg.toDirectSign(),
      memo: "",
      pubKey: publicKey,
      sequence: parseInt(aliceAccount.account.base_account.sequence, 10),
      accountNumber: parseInt(
        aliceAccount.account.base_account.account_number,
        10
      ),
      chainId: chainId,
    });

    txRaw.setSignaturesList([""]);
    simulationResponse = await grpcClient.simulate(txRaw);
  }
  let amount = new BigNumberInBase(500000001)
    .times(parseInt((simulationResponse.gasInfo.gasUsed * 1.3).toString()))
    .toString();
  let gas = parseInt(
    (simulationResponse.gasInfo.gasUsed * 1.3).toString()
  ).toString();

  const { signBytes, txRaw } = createTransaction({
    message: storeCodeMsg.toDirectSign(),
    memo: "",
    fee: {
      amount: [
        {
          amount: amount,
          denom: "route",
        },
      ],
      gas: gas,
    },
    pubKey: publicKey,
    sequence: parseInt(aliceAccount.account.base_account.sequence, 10),
    accountNumber: parseInt(
      aliceAccount.account.base_account.account_number,
      10
    ),
    chainId: chainId,
  });

  /** Sign transaction */
  const signature = await privateKey.sign(signBytes);

  /** Append Signatures */
  txRaw.setSignaturesList([signature]);
  /** Broadcast transaction */
  let txxResponse = await restClient.broadcast(txRaw);
  let txResponse = await restClient.waitTxBroadcast(txxResponse.txhash);
  const parsedLogs = parseRawLog(txResponse.raw_log);

  const codeIdAttr =
    typeof parsedLogs === "string"
      ? { value: "null" }
      : logs.findAttribute(parsedLogs, "store_code", "code_id");

  return codeIdAttr.value;
};
