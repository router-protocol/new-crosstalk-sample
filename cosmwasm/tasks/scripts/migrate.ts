import {
    getEndpointsForNetwork,
    PrivateKey,
    privateKeyToPublicKeyBase64,
    ChainRestAuthApi,
    createTransaction,
    BigNumberInBase,
    TxRestClient,
    Network,
    TxGrpcClient,
    TxClientSimulateResponse,
    MsgMigrateContract,
  } from "@routerprotocol/router-chain-sdk-ts";
  import dotenv from "dotenv";
  import { parseRawLog } from "@cosmjs/stargate/build/logs";
  import { logs } from "@cosmjs/stargate";
  dotenv.config();
  
  let network = Network.AlphaDevnet;
  if (process.env.NETWORK == "devnet") {
    network = Network.Devnet
  } else if (process.env.NETWORK == "testnet") {
    network = Network.Testnet
  } else if (process.env.NETWORK == "mainnet") {
    network = Network.Mainnet
  }
  const privateKeyHash = process.env.PRIVATE_KEY;
  const chainId = process.env.CHAIN_ID;
  if (!chainId) {
    throw new Error("Please set your CHAIN_ID in the .env file");
  }
  
  if (!privateKeyHash) {
    throw new Error("Please set your PRIVATE_KEY in the .env file");
  }
  
  const endpoint = getEndpointsForNetwork(network);
  const privateKey = PrivateKey.fromPrivateKey(privateKeyHash);
  
  const alice = privateKey.toBech32();
  
  const publicKey = privateKeyToPublicKeyBase64(
    Buffer.from(privateKeyHash, "hex")
  );
  
  const restClient = new TxRestClient(endpoint.lcdEndpoint);
  const grpcClient = new TxGrpcClient(endpoint.grpcEndpoint);

  export const migrateContract = async function (contractAddr: string, codeId: number, msg: Object): Promise<readonly logs.Log[]> {
    /** Get Faucet Accounts details */
    const aliceAccount = await new ChainRestAuthApi(
      endpoint.lcdEndpoint
    ).fetchAccount(alice);
    const executeContractMsg = MsgMigrateContract.fromJSON({
      senderAddress: alice,
      codeId: codeId,
      contractAddress: contractAddr,
      msg: msg
    });
  
    let simulationResponse: TxClientSimulateResponse;
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
  
    let amount = new BigNumberInBase(500000001)
      .times(
        parseInt(
          (
            simulationResponse.gasInfo.gasUsed * 1.3
          ).toString()
        )
      )
      .toString();
    let gas = parseInt(
      (
        simulationResponse.gasInfo.gasUsed * 1.3
      ).toString()
    ).toString();
    console.log(amount, gas)
  
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
    const parsedLogs = parseRawLog(txResponse.raw_log)
  
    return parsedLogs;
  }
  
  