//SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "evm-gateway-contract/contracts/IGateway.sol";
import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";

/// @title PingPong
/// @author Yashika Goyal
/// @notice This is a cross-chain ping pong smart contract to demonstrate how one can
/// utilise Router CrossTalk for cross-chain transactions.
contract PingPong is ICrossTalkApplication {
  // instance of the Router's gateway contract
  IGateway public gatewayContract;

  // greeting we will be setting when we send a cross-chain request
  string public greeting;

  // event nonce received when we create a cross-chain request
  // we will use this to verify whether the tx was executed on the 
  // dest chain when we get the acknowledgement back from the destination chain.
  uint64 public lastEventIdentifier;

  // gas limit required to handle the cross-chain request on the destination chain.
  uint64 public destGasLimit;

  // gas limit required to handle the acknowledgement received on the source 
  // chain back from the destination chain.
  uint64 public ackGasLimit;

  // custom error so that we can emit a custom error message
  error CustomError(string message);

  // events we will emit while handling acknowledgement 
  event ExecutionStatus(uint64 eventIdentifier, bool isSuccess);
  event ReceivedSrcChainIdAndType(uint64 chainType, string chainID);

  constructor(
    address payable gatewayAddress,
    uint64 _destGasLimit,
    uint64 _ackGasLimit
  ) {
    gatewayContract = IGateway(gatewayAddress);
    destGasLimit = _destGasLimit;
    ackGasLimit = _ackGasLimit;
  }

  /// @notice function to generate a cross-chain request to ping a destination chain contract.
  /// @param chainType chain type of the destination chain.
  /// @param chainId chain ID of the destination chain in string.
  /// @param destGasPrice gas price of the destination chain.
  /// @param ackGasPrice gas price of the source chain.
  /// @param destinationContractAddress contract address of the contract that will handle this
  /// request on the destination chain(in bytes format).
  /// @param str string we will be sending as greeting to the destination chain.
  /// @param expiryDurationInSeconds expiry duration of the request in seconds. After this time,
  /// if the request has not already been executed, it will fail on the destination chain.
  /// If you don't want to provide any expiry duration, send type(uint64).max in its place.
  function pingDestination(
    uint64 chainType,
    string memory chainId,
    uint64 destGasPrice,
    uint64 ackGasPrice,
    address destinationContractAddress,
    string memory str,
    uint64 expiryDurationInSeconds
  ) public payable {
    // creating the payload to be sent to the destination chain
    bytes memory payload = abi.encode(str);
    // creating the expiry timestamp
    uint64 expiryTimestamp = uint64(block.timestamp) + expiryDurationInSeconds;

    // creating an array of destination contract addresses in bytes
    bytes[] memory addresses = new bytes[](1);
    addresses[0] = toBytes(destinationContractAddress);

    // creating an array of payloads to be sent to respective destination contracts
    bytes[] memory payloads = new bytes[](1);
    payloads[0] = payload;

    // sending a cross-chain request
    _pingDestination(
      expiryTimestamp,
      destGasPrice,
      ackGasPrice,
      chainType,
      chainId,
      payloads,
      addresses
    );
  }

  function _pingDestination(
    uint64 expiryTimestamp,
    uint64 destGasPrice,
    uint64 ackGasPrice,
    uint64 chainType,
    string memory chainId,
    bytes[] memory payloads,
    bytes[] memory addresses
  ) internal {
    // Calling the requestToDest function on the Router's Gateway contract to generate a 
    // cross-chain request and storing the nonce returned into the lastEventIdentifier.
    lastEventIdentifier = gatewayContract.requestToDest(
      expiryTimestamp,
      false,
      Utils.AckType.ACK_ON_SUCCESS,
      Utils.AckGasParams(ackGasLimit, ackGasPrice),
      Utils.DestinationChainParams(
        destGasLimit,
        destGasPrice,
        chainType,
        chainId
      ),
      Utils.ContractCalls(payloads, addresses)
    );
  }

  /// @notice function to handle the cross-chain request received from some other chain.
  /// @param srcContractAddress address of the contract on source chain that initiated the request.
  /// @param payload the payload sent by the source chain contract when the request was created.
  /// @param srcChainId chain ID of the source chain in string.
  /// @param srcChainType chain type of the source chain.
  function handleRequestFromSource(
    bytes memory srcContractAddress,
    bytes memory payload,
    string memory srcChainId,
    uint64 srcChainType
  ) external override returns (bytes memory) {
    require(msg.sender == address(gatewayContract));

    string memory sampleStr = abi.decode(payload, (string));

    if (
      keccak256(abi.encodePacked(sampleStr)) == keccak256(abi.encodePacked(""))
    ) {
      revert CustomError("String should not be empty");
    }

    greeting = sampleStr;
    return abi.encode(srcChainId, srcChainType);
  }

  /// @notice function to handle the acknowledgement received from the destination chain 
  /// back on the source chain.
  /// @param eventIdentifier event nonce which is received when we create a cross-chain request
  /// We can use it to keep a mapping of which nonces have been executed and which did not.
  /// @param execFlags an array of boolean values suggesting whether the calls were successfully
  /// executed on the destination chain. 
  /// @param execData an array of bytes returning the data returned from the handleRequestFromSource
  /// function of the destination chain.
  function handleCrossTalkAck(
    uint64 eventIdentifier,
    bool[] memory execFlags,
    bytes[] memory execData
  ) external override {
    require(lastEventIdentifier == eventIdentifier, "wrong event identifier");
    bytes memory _execData = abi.decode(execData[0], (bytes));

    (string memory chainID, uint64 chainType) = abi.decode(
      _execData,
      (string, uint64)
    );

    emit ExecutionStatus(eventIdentifier, execFlags[0]);
    emit ReceivedSrcChainIdAndType(chainType, chainID);
  }


  /// @notice function to convert address to bytes
  function toBytes(address a) public pure returns (bytes memory b) {
    assembly {
      let m := mload(0x40)
      a := and(a, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
      mstore(add(m, 20), xor(0x140000000000000000000000000000000000000000, a))
      mstore(0x40, add(m, 52))
      b := m
    }
  }
}
