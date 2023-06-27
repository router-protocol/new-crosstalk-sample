# Router's NEAR Gateway Contracts

Router NEAR Gateway contract will bridge NEAR chain with the Router Chain.
We can deploy this gateway contract on NEAR chain.

## Overview

The NEAR gateway contract implements three funcitonlity.

1. Send request to the Router chain or any other destination chain.
2. Handle request from the Router chain or any other destination chain.
3. Update Validator set on the gateway contact.

## Please use the following instruction to setup, test and deploy the project

## Send Request to the Router Chain

To send request to router chain the user contract need to call the following function
and needs to provide the bridge contract address & payload bytes

```sh
# Gateway contract address variable
Gateway public gatewayContract;

# User/ Application contract constructor
constructor(address gatewayAddress) {
    gatewayContract = Gateway(gatewayAddress);
}

# example of send request to the Router chain
function sendRequestToRouter(bytes memory payload, string memory routerBridgeContract) public payable {
    # implement the business logic
    gatewayContract.requestToRouter(payload, routerBridgeContract);
}
```

## Handle Request from the Router

To handle request coming from the router chain, the user contract needs to implement
the following function in their contract

```sh
function handleRequestFromRouter(string memory sender, bytes memory payload) external {
    # implement the business logic
}
```

In case of state update from the _requestFromRouter_ function we are emitting the following event

```sh
# This is OutBound Request Acknowledgement event
event EventOutboundAck(
    uint256 ChainType,
    string  ChainId,
    uint256 OutboundTxNonce,
    bytes   contractAckResponses,
    uint8   exeCode,
    bool    status
);
```

Currently we are emitting this outbound acknowlegdement event in two cases only.
The ChainType, ChainId, and OutboundTxNonce will have same values in all cases.

- When the txn is valid but It is getting executed past the timeout.
  In this scenario, we will update the nonce mapping to 1 as executed and event will have the following values

  ```
  event EventOutboundAck(
      ChainType,
      ChainId,
      OutboundTxNonce,
      "",
      3,
      false
  );
  ```

- When the txn is valid and executed its handler calls to user contract
  In this scenario, we will update the nonce mapping to 1 as executed and event will have the following values
  ```
  event EventOutboundAck(
      ChainType,
      ChainId,
      OutboundTxNonce,
      data,
      0,
      success
  );
  ```
  Here, data and success values are coming from the _handlerExecuteCalls_ funciton.
  Data bytes can be decoded according to the success value. If it is true, then it will be
  array of bool values and if it is false, then it will string value.

## Update Validator Set

This is used to update validator set on the gateway contract.
This will be called by the router chain validator set only.

## Setup

```
cd router-gateway-contracts/substrate
rustup toolchain install nightly-2022-08-15
rustup target add wasm32-unknown-unknown --toolchain nightly-2022-08-15
rustup component add rust-src --toolchain nightly-2022-08-15
cargo +nightly-2022-08-15 contract build
```

## Run Tests

Use the following commands to run the test cases:

```
cargo +nightly-2022-08-15 contract test
```

## Deploy Gateway Contract on live network

Add gateway contract constructor arguments in args.json

```
cd router-gateway-contracts/substrate
npx hardhat deploy --network <network>
```

## Verify GateWay Contract on a network

```
cd router-gateway-contracts/substrate
npx hardhat verify --constructor-args <args-file-path> <gateway-contract-address> --network <network-name>
```

Example:-

```
npx hardhat verify --constructor-args scripts/arguments.js 0x610aEe9387488398c25Aca6aDFBac662177DB24D --network polygonMumbai
```
