# Router Substrate Gateway Contracts

## Overview

The Router Substrate Gateway contract plays a crucial role in facilitating seamless data transfer and interaction between substrate chains and other chains that are integrated with the router chain. By acting as a reliable and efficient bridge, it enables the smooth transfer of data, assets, and functionalities across different chains.

One of the remarkable advantages of this gateway contract is its versatility. It offers unparalleled flexibility, allowing us to deploy it on any substrate compatible chain. This means that regardless of the specific substrate chain being used, we can effortlessly integrate and establish a connection with the router chain, ensuring the seamless flow and exchange of information.

With the deployment of the Router Substrate Gateway contract, we can unlock a whole new level of interoperability and connectivity among various chains. This not only enhances the overall efficiency and effectiveness of our network, but also opens up exciting possibilities for collaboration, innovation, and scalability.

## Prerequisites

Before you can contribute to this project, you must have the following installed:
* [Rust & Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
* [Cargo-Contract](https://github.com/paritytech/cargo-contract#installation)
* [Node.Js](https://nodejs.org/)
* [Yarn](https://yarnpkg.com/)

## Getting Started
To get started with this project, you should first clone the repository:

```sh
git clone git@github.com:router-protocol/router-gateway-contracts.git
```

Once you have cloned the repository, you can navigate to the substrate directory and run the following command to install the project dependencies:

```sh
yarn
```

## Build Contrract
To build contract you can pass gateway, route_token or gateway or either, passing other than that will through error

```sh
sh build.sh --build gateway dapp route_token
```

## Generate Types

Create types for deploying and interacting with the contract. This will create [type-chain](https://learn.brushfam.io/docs/Typechain) types.

```sh
yarn types
```

## Deploy Gateway

```sh
ts-node ./scripts/deploy.ts --net "aleph-testnet" --type "gateway" --routetoken "RouteToken_Address(Optional)" --env "testnet or mainnet oralpha"
```

## Deploy Dapp

```sh
ts-node ./scripts/deploy.ts --net "aleph-testnet" --type "dapp" --gateway Gateway Address --env "testnet or mainnet oralpha"
```

## Interact With Gateway Vai Dapp

Note: To interact add gateway.json to path "./scripts/config/gateway.json" and follow same structure from "./scripts/config/gateway.example.json"

### ISend

```sh
 s-node ./scripts/gateway.ts --dapp "" --gateway "" --type "isend" --net aleph-testnet
```

### SetDappMetadata

```sh
ts-node ./scripts/gateway.ts --dapp "" --gateway "" --type "set_dapp_metadata" --net aleph-testnet
```

### IReceive

```sh
ts-node ./scripts/gateway.ts --dapp "" --gateway "" --type "ireceive" --net aleph-testnet
```

### UpdateValsetUpdate

```sh
ts-node ./scripts/gateway.ts --dapp "" --gateway "" --type "update_valset" --net aleph-testnet
```

Note: While interacting with dapp from other chain to aleph zero pass hex formated address instead of SS558 format

## To Get hex address from SS558

```ts
function ss558AccountIdToHexEncodedString(
  api: ApiPromise,
  accountId: string
): string {
  return (
    "0x" +
    Buffer.from(
      api.registry.createType("AccountId", accountId).toU8a()
    ).toString("hex")
  );
}
```

## To Get SS558 address from Hex

```ts
function ss558AccountIdFromHexEncodedString(
  api: ApiPromise,
  accountId: string
): string {
  return api.registry.createType("AccountId", accountId).toString();
}
```


## Contributing

If you would like to contribute to this project, please fork the repository and submit a pull request. All contributions are welcome!

