import fs from "fs";
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Signer, SystemProgram } from "@solana/web3.js";
import { ethers } from "ethers";
const {
  NetworkEndpoints,
  getEndpointsForNetwork,
  Network,
  ChainRestAuthApi,
  createTransaction,
  BigNumberInBase,
  getRouterAddress,
  PrivateKey,
  TxRestClient,
  privateKeyToPublicKeyBase64,
  MsgApproveFeepayerRequest,
  TxClientSimulateResponse,
  TxGrpcClient,
  getChainInfoForNetwork,
  MsgSend,
  DEFAULT_STD_FEE,
  getEthereumChainIdForNetwork,
} = require("@routerprotocol/router-chain-sdk-ts");
const { parseRawLog } = require("@cosmjs/stargate/build/logs");

export function parseCommandLineArgs(): any {
  const parsedArgs: any = {};

  const args: string[] = process.argv;
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg.startsWith("--")) parsedArgs[arg.slice(2)] = args[i + 1];
  }
  return parsedArgs;
}

export const CLUSTOR_CONFIG = {
  "solana-devnet": {
    "url": "https://api.devnet.solana.com",
    "chainId": "solana-devnet",
  },
  "solana": {
    "url": "https://api.mainnet-beta.solana.com",
    "chainId": "solana",
  },
};

export function getClustorConfig(network: string): any {
  if (
    network != "solana-devnet" &&
    network != "solana-testnet" &&
    network != "solana"
  )
    throw new Error(
      "Invalid Clustor, Pass solana-devnet, solana-testnet or solana-mainnet"
    );
  return CLUSTOR_CONFIG[network];
}

type PDA = {
  account: PublicKey;
  bump: number;
};

export type GatewayPdas = {
  gatewayAccount: PDA;
  gatewayAuthority: PDA;
  currentValsetAccount: PDA;
  eventAuthority: PDA;
};

export type PingPongPdas = {
  pingPongAccount: PDA;
  eventAuthority: PDA;
  dappAccount?: PDA;
};

export const PACKET_SEED_PREFIX = anchor.utils.bytes.utf8.encode("PA_");
export const VALSET_SEED_PREFIX = anchor.utils.bytes.utf8.encode("VA_");
export const CURRENT_VALSET_SEED =
  anchor.utils.bytes.utf8.encode("current_valset");
export const GATEWAY_ACCOUNT_SEED =
  anchor.utils.bytes.utf8.encode("gateway_account");
export const AUTHORITY_SEED = anchor.utils.bytes.utf8.encode("authority");
export const EVENT_AUTHORITY_SEED =
  anchor.utils.bytes.utf8.encode("__event_authority");

export function getGatewayPdas(
  gatewayProgramId: anchor.web3.PublicKey
): GatewayPdas {
  const currentValset = anchor.web3.PublicKey.findProgramAddressSync(
    [VALSET_SEED_PREFIX, CURRENT_VALSET_SEED],
    gatewayProgramId
  );
  const gatewayAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [GATEWAY_ACCOUNT_SEED],
    gatewayProgramId
  );
  const gatewayAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [AUTHORITY_SEED],
    gatewayProgramId
  );
  const eventAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [EVENT_AUTHORITY_SEED],
    gatewayProgramId
  );
  return {
    gatewayAccount: {
      account: gatewayAccount[0],
      bump: gatewayAccount[1],
    },
    gatewayAuthority: {
      account: gatewayAuthority[0],
      bump: gatewayAuthority[1],
    },
    currentValsetAccount: {
      account: currentValset[0],
      bump: currentValset[1],
    },
    eventAuthority: {
      account: eventAuthority[0],
      bump: eventAuthority[1],
    },
  };
}

export function getPingPongPdas(
  pingPongProgramId: anchor.web3.PublicKey,
  gatewayProgramId?: anchor.web3.PublicKey
): PingPongPdas {
  const pingPongAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("ping_pong")],
    pingPongProgramId
  );
  const eventAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [EVENT_AUTHORITY_SEED],
    pingPongProgramId
  );
  let dappAccount = null;
  if (gatewayProgramId)
    dappAccount = anchor.web3.PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.utf8.encode("dapp_account"),
        pingPongAccount[0].toBuffer(),
      ],
      gatewayProgramId
    );
  return {
    pingPongAccount: {
      account: pingPongAccount[0],
      bump: pingPongAccount[1],
    },
    eventAuthority: {
      account: eventAuthority[0],
      bump: eventAuthority[1],
    },
    dappAccount: gatewayProgramId
      ? {
          account: dappAccount[0],
          bump: dappAccount[1],
        }
      : null,
  };
}

export function getRequestMetadataEncodePacked(
  destGasLimit: number,
  destGasPrice: number,
  ackGasLimit: number,
  ackGasPrice: number,
  relayerFees: number,
  ackType: number,
  isReadCall: boolean,
  asmAddress: string
): string {
  return ethers.solidityPacked(
    [
      "uint64",
      "uint64",
      "uint64",
      "uint64",
      "uint128",
      "uint8",
      "bool",
      "string",
    ],
    [
      destGasLimit,
      destGasPrice,
      ackGasLimit,
      ackGasPrice,
      relayerFees,
      ackType,
      isReadCall,
      asmAddress,
    ]
  );
}

export function getRandomInt(min: number, max: number): number {
  min = Math.ceil(min);
  max = Math.floor(max);
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

export async function getEventByName(
  provider: anchor.Provider,
  signature: string,
  program: anchor.Program,
  eventName: string
) {
  const events = await readEvents(provider, signature, [program]);
  return getEvent(events, program.programId, eventName);
}

export async function readEvents(
  provider: anchor.Provider,
  txSignature: string,
  programs
) {
  const txResult = await provider.connection.getTransaction(txSignature, {
    commitment: "confirmed",
    maxSupportedTransactionVersion: 0,
  });

  let eventAuthorities = new Map();
  for (const program of programs) {
    eventAuthorities.set(
      program.programId.toString(),
      anchor.web3.PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("__event_authority")],
        program.programId
      )[0]
    );
  }

  const accountKeys = txResult.transaction.message.getAccountKeys
    ? txResult.transaction.message.getAccountKeys()?.staticAccountKeys
    : //@ts-ignore
      txResult.transaction.message.accountKeys;

  let events = [];
  for (const ixBlock of txResult.meta.innerInstructions) {
    for (const ix of ixBlock.instructions) {
      for (const program of programs) {
        const programStr = program.programId.toString();
        if (
          ix.accounts.length === 1 &&
          accountKeys[ix.programIdIndex].toString() === programStr &&
          accountKeys[ix.accounts[0]].toString() ===
            eventAuthorities.get(programStr).toString()
        ) {
          const ixData = anchor.utils.bytes.bs58.decode(ix.data);
          const eventData = anchor.utils.bytes.base64.encode(ixData.slice(8));
          const event = program.coder.events.decode(eventData);
          events.push({
            program: program.programId,
            data: event.data,
            name: event.name,
          });
        }
      }
    }
  }
  return events;
}

export function getEvent(
  events: any[],
  program: anchor.web3.PublicKey,
  eventName: string
) {
  for (const event of events) {
    if (
      event.name === eventName &&
      program.toString() === event.program.toString()
    ) {
      return event.data;
    }
  }
  throw new Error("Event " + eventName + " not found");
}

export async function decodeRequestPacket(
  provider: anchor.Provider,
  requestPacket: PublicKey
) {
  const requestPacketData = Uint8Array.from(
    (await provider.connection.getAccountInfo(requestPacket, "confirmed")).data
  ).slice(8);

  let offset = 33;
  const length = new anchor.BN(
    requestPacketData.slice(offset, offset + 4).reverse()
  ).toNumber();
  offset += 4;

  const dstContractFromPacketLength = new anchor.BN(
    requestPacketData.slice(offset, offset + 4).reverse()
  ).toNumber();
  offset += 4;

  const dstContractFromPacket = ethers.toUtf8String(
    requestPacketData.slice(offset, offset + dstContractFromPacketLength)
  );

  offset += dstContractFromPacketLength;
  const payloadFromPacketLength = new anchor.BN(
    requestPacketData.slice(offset, offset + 4).reverse()
  ).toNumber();
  offset += 4;

  const payloadFromPacket = ethers.hexlify(
    requestPacketData.slice(offset, offset + payloadFromPacketLength)
  );
  const [requestId, message] = new ethers.AbiCoder().decode(
    ["uint256", "string"],
    payloadFromPacket
  );
  return {
    dstContractFromPacket,
    requestId,
    message,
  };
}

export function getStrSolanaHandlerAddress(
  addresses: anchor.web3.PublicKey[]
): string {
  let bytes: number[] = [];
  addresses.map((address) => (bytes = [...bytes, ...address.toBytes()]));
  return ethers.hexlify(Uint8Array.from(bytes));
}

export function getPubSolanaHandlerAddress(
  str: string
): anchor.web3.PublicKey[] {
  const bytes = ethers.getBytes(str);
  const addresses: any[] = [];
  for (let idx = 0; idx < bytes.length / 32; idx++)
    addresses.push(
      new anchor.web3.PublicKey(bytes.slice(idx * 32, (idx + 1) * 32))
    );
  return addresses;
}

export async function feeApprovalMsg(
  privateKeyHash: Uint8Array,
  srcChainId: string,
  dAppAddress: string,
  network: typeof Network
) {
  const endpoint = getEndpointsForNetwork(network);
  const chainId = getChainInfoForNetwork(network).chainId;
  const privateKey = PrivateKey.fromPrivateKey(privateKeyHash);
  const alice = privateKey.toBech32();
  const publicKey = privateKeyToPublicKeyBase64(Buffer.from(privateKeyHash));
  const restClient = new TxRestClient(endpoint.lcdEndpoint);
  const grpcClient = new TxGrpcClient(endpoint.grpcEndpoint);
  const aliceAccount = await new ChainRestAuthApi(
    endpoint.lcdEndpoint
  ).fetchAccount(alice);
  const message = MsgApproveFeepayerRequest.fromJSON({
    feepayer: alice,
    chainid: srcChainId,
    dappaddresses: dAppAddress,
  });
  let simulationResponse: any;
  {
    let { txRaw } = createTransaction({
      message: message.toDirectSign(),
      memo: "",
      // fee: DEFAULT_STD_FEE,
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
  let txResponse = await restClient.waitTxBroadcast(
    txxResponse.txhash,
    60000000000000
  );
  const parsedLogs = parseRawLog(txResponse.raw_log);
  return { hash: txxResponse.txhash, logs: parsedLogs };
}
