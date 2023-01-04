// SPDX-License-Identifier: Unlicensed
pragma solidity >=0.8.0 <0.9.0;

import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";
import "./CrossTalkUtils.sol";

contract PingPong is ICrossTalkApplication {
  address public gatewayContract;
  string public greeting;
  uint64 public lastEventIdentifier;
  uint64 public destGasLimit;
  uint64 public ackGasLimit;

  error CustomError(string message);
  event ExecutionStatus(uint64 eventIdentifier, bool isSuccess);
  event ReceivedSrcChainIdAndType(uint64 chainType, string chainID);

  constructor(
    address payable gatewayAddress,
    uint64 _destGasLimit,
    uint64 _ackGasLimit
  ) {
    gatewayContract = gatewayAddress;
    destGasLimit = _destGasLimit;
    ackGasLimit = _ackGasLimit;
  }

  function pingDestination(
    uint64 chainType,
    string memory chainId,
    uint64 destGasPrice,
    uint64 ackGasPrice,
    bytes memory destinationContractAddress,
    string memory str,
    uint64 expiryDurationInSeconds
  ) public payable {
    bytes memory payload = abi.encode(str);
    uint64 expiryTimestamp = uint64(block.timestamp) + expiryDurationInSeconds;
    Utils.DestinationChainParams memory destChainParams = Utils
      .DestinationChainParams(destGasLimit, destGasPrice, chainType, chainId);

    Utils.AckType ackType = Utils.AckType.ACK_ON_SUCCESS;
    Utils.AckGasParams memory ackGasParams = Utils.AckGasParams(
      ackGasLimit,
      ackGasPrice
    );

    lastEventIdentifier = CrossTalkUtils.singleRequestWithAcknowledgement(
      gatewayContract,
      expiryTimestamp,
      ackType,
      ackGasParams,
      destChainParams,
      destinationContractAddress,
      payload
    );
  }

  function handleRequestFromSource(
    bytes memory, //srcContractAddress
    bytes memory payload,
    string memory srcChainId,
    uint64 srcChainType
  ) external override returns (bytes memory) {
    require(msg.sender == gatewayContract);

    string memory sampleStr = abi.decode(payload, (string));

    if (
      keccak256(abi.encodePacked(sampleStr)) == keccak256(abi.encodePacked(""))
    ) {
      revert CustomError("String should not be empty");
    }
    greeting = sampleStr;
    return abi.encode(srcChainId, srcChainType);
  }

  function handleCrossTalkAck(
    uint64 eventIdentifier,
    bool[] memory execFlags,
    bytes[] memory execData
  ) external override {
    require(lastEventIdentifier == eventIdentifier);
    bytes memory _execData = abi.decode(execData[0], (bytes));

    (string memory chainID, uint64 chainType) = abi.decode(
      _execData,
      (string, uint64)
    );

    // emits the event identifier and true as execFlags[0]
    emit ExecutionStatus(eventIdentifier, execFlags[0]);
    // emits the source chain Id and type that it gets back from the dest chain
    emit ReceivedSrcChainIdAndType(chainType, chainID);
  }
}
