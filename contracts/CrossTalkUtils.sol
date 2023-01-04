// SPDX-License-Identifier: Unlicensed
pragma solidity ^0.8.4;

import "evm-gateway-contract/contracts/IGateway.sol";
import "evm-gateway-contract/contracts/Utils.sol";

/// @title CrossTalkUtils
/// @author Router Protocol
/// @notice This contract can be used to abstract the complexities while using the
/// Router CrossTalk framework.
library CrossTalkUtils {
  /// @notice Fuction to get whether the calls were executed on the destination chain.
  /// @param execFlags Array of boolean flags which indicate the execution status of calls on dest chain.
  /// @return boolean value indicating whether the calls were successfully executed on destination chain.
  function getTxStatusForAtomicCall(
    bool[] calldata execFlags
  ) internal pure returns (bool) {
    return execFlags[execFlags.length - 1] == true;
  }

  /// @notice Fuction to get the index of call out of an array of calls that failed on the destination chain.
  /// @param execFlags Array of boolean flags which indicate the execution status of calls on dest chain.
  /// @return index of call that failed
  function getTheIndexOfCallFailure(
    bool[] calldata execFlags
  ) internal pure returns (uint8) {
    require(getTxStatusForAtomicCall(execFlags), "No calls failed");

    for (uint8 i = 0; i < execFlags.length; i++) {
      if (execFlags[i] == false) {
        return i;
      }
    }

    return 0;
  }

  /// @notice Function to convert address to bytes
  /// @param addr address to be converted
  /// @return b bytes pertaining to address addr
  function toBytes(address addr) internal pure returns (bytes memory b) {
    assembly {
      let m := mload(0x40)
      addr := and(addr, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
      mstore(
        add(m, 20),
        xor(0x140000000000000000000000000000000000000000, addr)
      )
      mstore(0x40, add(m, 52))
      b := m
    }
  }

  /// @notice Function to convert bytes to address
  /// @param _bytes bytes to be converted
  /// @return addr address pertaining to the bytes
  function toAddress(bytes memory _bytes) internal pure returns (address addr) {
    bytes20 srcTokenAddress;
    assembly {
      srcTokenAddress := mload(add(_bytes, 0x20))
    }
    addr = address(srcTokenAddress);
  }

  /// @notice Function to send a single request without acknowledgement to the destination chain.
  /// @dev You will be able to send a single request to a single contract on the destination chain and
  /// you don't need the acknowledgement back on the source chain.
  /// @param gatewayContract address of the gateway contract.
  /// @param expiryTimestamp timestamp when the call expires. If this time passes by, the call will fail
  /// on the destination chain. If you don't want to add an expiry timestamp, set it to zero.
  /// @param destChainParams dest chain params include the destChainType, destChainId, the gas limit
  /// required to execute handler function on the destination chain and the gas price of destination chain.
  /// @param destinationContractAddress Contract address (in bytes format) of the contract which will be
  /// called on the destination chain which will handle the payload.
  /// @param payload abi encoded data that you want to send to the destination chain.
  /// @return Returns the nonce from the gateway contract.
  function singleRequestWithoutAcknowledgement(
    address gatewayContract,
    uint64 expiryTimestamp,
    Utils.DestinationChainParams memory destChainParams,
    bytes memory destinationContractAddress,
    bytes memory payload
  ) internal returns (uint64) {
    if (expiryTimestamp == 0) {
      expiryTimestamp = type(uint64).max;
    }

    bytes[] memory addresses = new bytes[](1);
    addresses[0] = destinationContractAddress;
    bytes[] memory payloads = new bytes[](1);
    payloads[0] = payload;

    return
      IGateway(gatewayContract).requestToDest(
        expiryTimestamp,
        false,
        Utils.AckType.NO_ACK,
        Utils.AckGasParams(0, 0),
        destChainParams,
        Utils.ContractCalls(payloads, addresses)
      );
  }

  /// @notice Function to send a single request with acknowledgement to the destination chain.
  /// @dev You will be able to send a single request to a single contract on the destination chain and
  /// you need the acknowledgement back on the source chain.
  /// @param gatewayContract address of the gateway contract.
  /// @param expiryTimestamp timestamp when the call expires. If this time passes by, the call will fail
  /// on the destination chain. If you don't want to add an expiry timestamp, set it to zero.
  /// @param ackType type of acknowledgement you want: ACK_ON_SUCCESS, ACK_ON_ERR, ACK_ON_BOTH.
  /// @param ackGasParams This includes the gas limit required for the execution of handler function for
  /// crosstalk acknowledgement on the source chain and the gas price of the source chain.
  /// @param destChainParams dest chain params include the destChainType, destChainId, the gas limit
  /// required to execute handler function on the destination chain and the gas price of destination chain.
  /// @param destinationContractAddress Contract address (in bytes format) of the contract which will be
  /// called on the destination chain which will handle the payload.
  /// @param payload abi encoded data that you want to send to the destination chain.
  /// @return Returns the nonce from the gateway contract.
  function singleRequestWithAcknowledgement(
    address gatewayContract,
    uint64 expiryTimestamp,
    Utils.AckType ackType,
    Utils.AckGasParams memory ackGasParams,
    Utils.DestinationChainParams memory destChainParams,
    bytes memory destinationContractAddress,
    bytes memory payload
  ) internal returns (uint64) {
    if (expiryTimestamp == 0) {
      expiryTimestamp = type(uint64).max;
    }

    bytes[] memory addresses = new bytes[](1);
    addresses[0] = destinationContractAddress;
    bytes[] memory payloads = new bytes[](1);
    payloads[0] = payload;

    return
      IGateway(gatewayContract).requestToDest(
        expiryTimestamp,
        false,
        ackType,
        ackGasParams,
        destChainParams,
        Utils.ContractCalls(payloads, addresses)
      );
  }

  /// @notice Function to send multiple requests without acknowledgement to multiple contracts on the
  /// destination chain.
  /// @dev You will be able to send multiple requests to multiple contracts on the destination chain and
  /// you don't need the acknowledgement back on the source chain.
  /// @param gatewayContract address of the gateway contract.
  /// @param expiryTimestamp timestamp when the call expires. If this time passes by, the call will fail
  /// on the destination chain. If you don't want to add an expiry timestamp, set it to zero.
  /// @param isAtomicCalls boolean value suggesting whether the calls are atomic. If true, either all the
  /// calls will be executed or none will be executed on the destination chain. If false, even if some calls
  /// fail, others will not be affected.
  /// @param destChainParams dest chain params include the destChainType, destChainId, the gas limit
  /// required to execute handler function on the destination chain and the gas price of destination chain.
  /// @param destinationContractAddresses Array of contract addresses (in bytes format) of the contracts
  /// which will be called on the destination chain which will handle the respective payloads.
  /// @param payloads Array of abi encoded data that you want to send to the destination chain.
  /// @return Returns the nonce from the gateway contract.
  function multipleRequestsWithoutAcknowledgement(
    address gatewayContract,
    uint64 expiryTimestamp,
    bool isAtomicCalls,
    Utils.DestinationChainParams memory destChainParams,
    bytes[] memory destinationContractAddresses,
    bytes[] memory payloads
  ) internal returns (uint64) {
    if (expiryTimestamp == 0) {
      expiryTimestamp = type(uint64).max;
    }

    return
      IGateway(gatewayContract).requestToDest(
        expiryTimestamp,
        isAtomicCalls,
        Utils.AckType.NO_ACK,
        Utils.AckGasParams(0, 0),
        destChainParams,
        Utils.ContractCalls(payloads, destinationContractAddresses)
      );
  }

  /// @notice Function to send multiple requests with acknowledgement to multiple contracts on the
  /// destination chain.
  /// @dev You will be able to send multiple requests to multiple contracts on the destination chain and
  /// you need the acknowledgement back on the source chain.
  /// @param gatewayContract address of the gateway contract.
  /// @param expiryTimestamp timestamp when the call expires. If this time passes by, the call will fail
  /// on the destination chain. If you don't want to add an expiry timestamp, set it to zero.
  /// @param isAtomicCalls boolean value suggesting whether the calls are atomic. If true, either all the
  /// calls will be executed or none will be executed on the destination chain. If false, even if some calls
  /// fail, others will not be affected.
  /// @param ackType type of acknowledgement you want: ACK_ON_SUCCESS, ACK_ON_ERR, ACK_ON_BOTH.
  /// @param ackGasParams This includes the gas limit required for the execution of handler function for
  /// crosstalk acknowledgement on the source chain and the gas price of the source chain.
  /// @param destChainParams dest chain params include the destChainType, destChainId, the gas limit
  /// required to execute handler function on the destination chain and the gas price of destination chain.
  /// @param destinationContractAddresses Array of contract addresses (in bytes format) of the contracts
  /// which will be called on the destination chain which will handle the respective payloads.
  /// @param payloads Array of abi encoded data that you want to send to the destination chain.
  /// @return Returns the nonce from the gateway contract.
  function multipleRequestsWithAcknowledgement(
    address gatewayContract,
    uint64 expiryTimestamp,
    bool isAtomicCalls,
    Utils.AckType ackType,
    Utils.AckGasParams memory ackGasParams,
    Utils.DestinationChainParams memory destChainParams,
    bytes[] memory destinationContractAddresses,
    bytes[] memory payloads
  ) internal returns (uint64) {
    if (expiryTimestamp == 0) {
      expiryTimestamp = type(uint64).max;
    }

    return
      IGateway(gatewayContract).requestToDest(
        expiryTimestamp,
        isAtomicCalls,
        ackType,
        ackGasParams,
        destChainParams,
        Utils.ContractCalls(payloads, destinationContractAddresses)
      );
  }
}
