import {
  getEndpointsForNetwork,
  PrivateKey,
  ChainRestAuthApi,
  createTransaction,
  BigNumberInBase,
  TxRestClient,
  Network,
  TxGrpcClient,
  MsgExecuteContract,
  getChainInfoForNetwork,
} from "@routerprotocol/router-chain-sdk-ts";
import dotenv from "dotenv";
dotenv.config();

export async function exec_msg(
  contractAddr: string,
  action: string,
  message: Object,
  nativeCoin: string,
  network: Network
) {
  const chainInfo = getChainInfoForNetwork(network);
  const chainId = chainInfo.chainId;
  const endpoint = getEndpointsForNetwork(network);

  const restClient = new TxRestClient(endpoint.lcdEndpoint);
  const grpcClient = new TxGrpcClient(endpoint.grpcEndpoint);

  if (!process.env.IS_MNEOMIC && process.env.PRIVATE_KEY) {
    throw new Error("Please set your PRIVATE_KEY or MNEOMIC in the .env file");
  }
  if (process.env.IS_MNEOMIC == "true" && !process.env.MNEOMIC)
    throw new Error("Please set your MNEOMIC or MNEOMIC in the .env file");

  const privateKey =
    process.env.IS_MNEOMIC == "true"
      ? //@ts-ignore
        PrivateKey.fromMnemonic(process.env.MNEOMIC)
      : //@ts-ignore
        PrivateKey.fromPrivateKey(process.env.PRIVATE_KEY);

  const alice = privateKey.toBech32();
  const publicKey = privateKey.toPublicKey().toBase64();

  /** Get Faucet Accounts details */
  const aliceAccount = await new ChainRestAuthApi(
    endpoint.lcdEndpoint
  ).fetchAccount(alice);

  let executeContractMsg;
  if (!nativeCoin) {
    executeContractMsg = MsgExecuteContract.fromJSON({
      sender: alice,
      action: action,
      contractAddress: contractAddr,
      msg: message,
    });
  } else {
    let nativeFunds = {
      denom: "route",
      amount: nativeCoin,
    };
    executeContractMsg = MsgExecuteContract.fromJSON({
      funds: nativeFunds,
      sender: alice,
      action: action,
      contractAddress: contractAddr,
      msg: message,
    });
  }

  let simulationResponse;
  {
    let { txRaw } = createTransaction({
      message: executeContractMsg.toDirectSign(),
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

  let gas = parseInt(
    (simulationResponse.gasInfo.gasUsed * 1.3).toString()
  ).toString();
  let amount = new BigNumberInBase(500000001).times(gas).toString();

  const { signBytes, txRaw } = createTransaction({
    message: executeContractMsg.toDirectSign(),
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
  return txResponse;
}
