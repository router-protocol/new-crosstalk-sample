import {
  ChainRestAuthApi,
  createTransaction,
  BigNumberInBase,
  PrivateKey,
  TxRestClient,
  privateKeyToPublicKeyBase64,
  MsgApproveFeepayerRequest,
  TxClientSimulateResponse,
  TxGrpcClient,
} from "@routerprotocol/router-chain-sdk-ts";
import dotenv from "dotenv";
import { getChainInfo, getEndpoints } from "./utils";
dotenv.config();

const endpoints = getEndpoints();
const chainInfo = getChainInfo();
export async function approve(srcChainId: string, dappAddress: string) {
  try {
    const privateKeyHash = process.env.FEE_PAYER_PRIVATE_KEY;

    if (!privateKeyHash) {
      throw new Error("Please set your FEE_PAYER_PRIVATE_KEY in the .env file");
    }

    const privateKey = PrivateKey.fromPrivateKey(privateKeyHash);

    const alice = privateKey.toBech32();

    const message = MsgApproveFeepayerRequest.fromJSON({
      feepayer: alice,
      chainid: srcChainId.trim(),
      dappaddresses: dappAddress.trim(),
    });

    const isApprovalComplete = await sendFeeApprovalTx(message, alice);
    if (isApprovalComplete) {
      console.log("Fee payer approval complete for ", srcChainId, dappAddress);
    }
  } catch (e) {
    console.error(
      "Fee payer approval failed for src chain and dapp address",
      srcChainId,
      dappAddress
    );
    console.log("error: ", e);
  }
}

async function sendFeeApprovalTx(
  message: MsgApproveFeepayerRequest,
  alice: string
) {
  try {
    const chainId = chainInfo.chainId;

    const privateKeyHash = process.env.FEE_PAYER_PRIVATE_KEY;

    if (!privateKeyHash) {
      throw new Error("Please set your FEE_PAYER_PRIVATE_KEY in the .env file");
    }

    const privateKey = PrivateKey.fromPrivateKey(privateKeyHash);

    const publicKey = privateKeyToPublicKeyBase64(
      Buffer.from(privateKeyHash, "hex")
    );

    const restClient = new TxRestClient(endpoints.lcdEndpoint);
    const grpcClient = new TxGrpcClient(endpoints.grpcEndpoint);

    /** Get Faucet Accounts details */
    const aliceAccount = await new ChainRestAuthApi(
      endpoints.lcdEndpoint
    ).fetchAccount(alice);

    let simulationResponse: TxClientSimulateResponse;
    {
      let { txRaw } = createTransaction({
        message: message.toDirectSign(),
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

    let amount = new BigNumberInBase(600000001)
      .times(parseInt((simulationResponse.gasInfo.gasUsed * 1.3).toString()))
      .toString();
    let gas = parseInt(
      (simulationResponse.gasInfo.gasUsed * 1.3).toString()
    ).toString();

    const { signBytes, txRaw } = createTransaction({
      message: message.toDirectSign(),
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
    await restClient.waitTxBroadcast(txxResponse.txhash);

    return true;
  } catch (e) {
    console.error("Error in fee payer approval: ", e);
    return false;
  }
}
