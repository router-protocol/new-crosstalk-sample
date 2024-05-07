import type { KeypairType } from "@polkadot/util-crypto/types";

type Chain = {
  type: KeypairType;
  id: string;
  name: string;
  rpcs: string[];
};

export const AlephZeroTestnet: Chain = {
  id: "aleph-testnet",
  name: "Aleph Zero Testnet",
  type: "sr25519", // or ed25519 :  *25519
  rpcs: ["wss://ws.test.azero.dev", "wss://aleph-zero-testnet-rpc.dwellir.com"],
};

const idmp = {
  "aleph-testnet": AlephZeroTestnet,
};

export const getNetwork = (id: string) => idmp[id];
