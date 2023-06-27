import {
  NetworkEndpoints,
  getEndpointsForNetwork,
  Network,
  getChainInfoForNetwork,
  ChainInfo,
  ChainGrpcMultiChainApi,
} from "@routerprotocol/router-chain-sdk-ts";
import dotenv from "dotenv";
dotenv.config();

const env = process.env.ROUTER_NETWORK;
let endpoints: NetworkEndpoints;
let chainInfo: ChainInfo;

if (env == "alpha") {
  endpoints = getEndpointsForNetwork(Network.AlphaDevnet);
  chainInfo = getChainInfoForNetwork(Network.AlphaDevnet);
} else if (env == "devnet") {
  endpoints = getEndpointsForNetwork(Network.Devnet);
  chainInfo = getChainInfoForNetwork(Network.Devnet);
} else if (env == "testnet") {
  endpoints = getEndpointsForNetwork(Network.Testnet);
  chainInfo = getChainInfoForNetwork(Network.Testnet);
} else if (env == "mainnet") {
  endpoints = getEndpointsForNetwork(Network.Mainnet);
  chainInfo = getChainInfoForNetwork(Network.Mainnet);
}

let nearChainId = "near-testnet";
if (env == "mainnet") {
  nearChainId = "near";
}

export function getEndpoints() {
  return endpoints;
}

export function getChainInfo() {
  return chainInfo;
}

export async function getGateway(
  chainId: String = nearChainId
): Promise<String> {
  try {
    const grpcEndpoint = endpoints.grpcEndpoint;
    const client = new ChainGrpcMultiChainApi(grpcEndpoint);

    const contractConfigs = await client.fetchAllContractConfig();
    const gateway = contractConfigs.contractconfigList.filter(
      (e) => e.chainid == chainId && e.contracttype == 0
    );

    return gateway[0].contractaddress;
  } catch (e) {
    console.log("Error in getting gateway from grpc: ", e);
    return "";
  }
}
