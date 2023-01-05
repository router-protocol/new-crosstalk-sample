// SPDX-License-Identifier: Unlicensed
pragma solidity >=0.8.0 <0.9.0;

import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";
import "@routerprotocol/router-crosstalk-utils/contracts/CrossTalkUtils.sol";

/// @title PingPong
/// @author Shivam Agrawal
/// @notice This is a cross-chain ping pong smart contract to demonstrate how one can
/// utilise Router CrossTalk for cross-chain transactions.
contract PingPong is ICrossTalkApplication {
  // address of the Router's gateway contract
  address public gatewayContract;

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
    gatewayContract = gatewayAddress;
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
    bytes memory destinationContractAddress,
    string memory str,
    uint64 expiryDurationInSeconds
  ) public payable {
    // creating the payload to be sent to the destination chain
    bytes memory payload = abi.encode(str);

    // creating the expiry timestamp
    uint64 expiryTimestamp = uint64(block.timestamp) + expiryDurationInSeconds;
    
    Utils.DestinationChainParams memory destChainParams = Utils
      .DestinationChainParams(destGasLimit, destGasPrice, chainType, chainId);

    Utils.AckType ackType = Utils.AckType.ACK_ON_SUCCESS;
    Utils.AckGasParams memory ackGasParams = Utils.AckGasParams(
      ackGasLimit,
      ackGasPrice
    );

    // Calling the singleRequestWithAcknowledgement function on the crosstalk utils library
    // to generate a cross-chain request and storing the nonce returned into the lastEventIdentifier
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
    // ensuring that only the gateway contract can send the cross-chain handling request
    require(msg.sender == gatewayContract);

    // decoding the payload we sent from the source chain
    string memory sampleStr = abi.decode(payload, (string));

    // checking the string received and throwing error if we received an empty string in the payload
    if (
      keccak256(abi.encodePacked(sampleStr)) == keccak256(abi.encodePacked(""))
    ) {
      revert CustomError("String should not be empty");
    }
    // setting the greeting if the string is non-empty
    greeting = sampleStr;

    // returning srcChainId and srcChainType which we  will receive when we get acknowledgemen
    // back on the source chain.
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
    // checking that the event identifier we received in acknowledgement is the same as what 
    // we received when we created a cross-chain request on the source chain.
    require(lastEventIdentifier == eventIdentifier);

    // decoding the execData in bytes first and then into the parameters we are expecting.
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
