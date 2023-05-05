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
    MsgInstantiateContract,
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

export const init_wasm_code = async function (codeId: string, label: string, instantiateMsg: string): Promise<string> {
    /** Get Faucet Accounts details */
    const aliceAccount = await new ChainRestAuthApi(
        endpoint.lcdEndpoint
    ).fetchAccount(alice);

    const initMsgObject = JSON.parse(instantiateMsg);
    const intantiateContractMsg = MsgInstantiateContract.fromJSON({
        sender: alice,
        admin: alice,
        codeId: parseInt(codeId),
        label: label,
        msg: initMsgObject,
    });
    let simulationResponse: TxClientSimulateResponse;
    {
        let { txRaw } = createTransaction({
            message: intantiateContractMsg.toDirectSign(),
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
        message: intantiateContractMsg.toDirectSign(),
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
    console.log(txxResponse);
    let txResponse = await restClient.waitTxBroadcast(txxResponse.txhash);
    console.log(`txResponse =>`, txResponse);
    const parsedLogs = parseRawLog(txResponse.raw_log)

    const contractAddressAttr =
        typeof parsedLogs === "string"
            ? { value: "null" }
            : logs.findAttribute(parsedLogs, "instantiate", "_contract_address");
    let initInfo = {
        contractAddress: contractAddressAttr.value,
        logs: parsedLogs,
        transactionHash: txxResponse.txhash,
    };
    console.log("store code info", initInfo);
    return contractAddressAttr.value;
}

// if (process.argv.length != 5) {
//     console.log(process.argv);
//     console.error('Expected three argument!');
//   process.exit(1);
// }

// codeId, label, instantiateMsg
// init_wasm_code(process.argv[2], process.argv[3], process.argv[4]);
