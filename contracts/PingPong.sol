// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "evm-gateway-contract/contracts/IGateway.sol";
import "evm-gateway-contract/contracts/IApplication.sol";
import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";
import "hardhat/console.sol";

contract HelloWorld is ICrossTalkApplication {
  IGateway public gatewayContract;
  string public greeting;
  uint64 public lastEventIdentifier;

  error CustomError(string message);

  constructor(address payable gatewayAddress) {
    gatewayContract = IGateway(gatewayAddress);
  }

  function pingDestination(
    uint64 chainType,
    string memory chainId,
    address destinationContractAddress,
    string memory str
  ) public payable {
    bytes memory payload = abi.encode(str);
    uint64 timestamp = 1681014199;
    bytes[] memory addresses = new bytes[](1);
    addresses[0] = toBytes(destinationContractAddress);
    bytes[] memory payloads = new bytes[](1);
    payloads[0] = payload;
    lastEventIdentifier = gatewayContract.multipleRequestsToDestWithAck(
      timestamp,
      false,
      Utils.AckType.ACK_ON_SUCCESS,
      Utils.AckGasParams(0, 0),
      Utils.DestinationChainParams(0, 0, chainType, chainId),
      Utils.ContractCalls(payloads, addresses)
    );
  }

  function handleRequestFromSource(
    bytes memory srcContractAddress,
    bytes memory payload,
    string memory srcChainId,
    uint64 srcChainType
  ) external override returns (bytes memory) {
    require(msg.sender == address(gatewayContract));
    require(address(this) == toAddress(srcContractAddress));

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
  ) external view override {
    console.log(eventIdentifier);
    for (uint i = 0; i < execFlags.length; i++) {
      console.log(execFlags[i]);
      console.logBytes(execData[i]);
    }

    require(lastEventIdentifier == eventIdentifier);
  }

  receive() external payable {}

  function toAddress(
    bytes memory _bytes
  ) internal pure returns (address contractAddress) {
    bytes20 srcTokenAddress;
    assembly {
      srcTokenAddress := mload(add(_bytes, 0x20))
    }
    contractAddress = address(srcTokenAddress);
  }

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
