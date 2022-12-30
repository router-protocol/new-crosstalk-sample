// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";
import "evm-gateway-contract/contracts/IGateway.sol";
import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";
import "hardhat/console.sol";

contract XERC1155 is ERC1155, ICrossTalkApplication {
  uint64 private _crossChainGasLimit;

  IGateway public gatewayContract;

  error CustomError(string message);

  constructor(
    string memory uri_,
    address payable gatewayAddress
  ) ERC1155(uri_) {
    gatewayContract = IGateway(gatewayAddress);
  }

  /**
   * @notice setCrossChainGasLimit Used to set CrossChainGas, this can only be set by CrossChain Admin or Admins
   * @param _gasLimit Amount of gasLimit that is to be set
   */
  function _setCrossChainGasLimit(uint64 _gasLimit) internal {
    _crossChainGasLimit = _gasLimit;
  }

  /**
   * @notice fetchCrossChainGasLimit Used to fetch CrossChainGas
   * @return crossChainGas that is set
   */
  function fetchCrossChainGasLimit() external view returns (uint64) {
    return _crossChainGasLimit;
  }

  function transferCrossChain(
    uint64 chainType,
    string memory chainId,
    address destinationContractAddress,
    uint64 _crossChainGasPrice,
    address _recipient,
    uint256[] memory _ids,
    uint256[] memory _amounts,
    bytes memory _data
  ) public payable {
    require(
      _recipient != address(0),
      "XERC1155: Recipient address cannot be null"
    );
    _burnBatch(msg.sender, _ids, _amounts);

    bytes memory payload = abi.encode(_recipient, _ids, _amounts, _data);

    gatewayContract.requestToDest(
      Utils.DestinationChainParams(
        _crossChainGasLimit,
        _crossChainGasPrice,
        chainType,
        chainId
      ),
      destinationContractAddress,
      payload
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

    (
      address _recipient,
      uint256[] memory _ids,
      uint256[] memory _amounts,
      bytes memory _data
    ) = abi.decode(payload, (address, uint256[], uint256[], bytes));

    if (
      keccak256(abi.encodePacked(_recipient, _ids, _amounts, _data)) ==
      keccak256(abi.encodePacked(""))
    ) {
      revert CustomError("Data should not be empty");
    }
    bool success = receiveCrossChain(_recipient, _ids, _amounts, _data);
    require(success == true, "Unsuccessful");
    return abi.encode(srcChainId, srcChainType);
  }

  function receiveCrossChain(
    address _recipient,
    uint256[] memory _ids,
    uint256[] memory _amounts,
    bytes memory _data
  ) internal returns (bool) {
    _mintBatch(_recipient, _ids, _amounts, _data);
    return true;
  }

  function handleCrossTalkAck(
    uint64, //eventIdentifier,
    bool[] memory, //execFlags,
    bytes[] memory //execData
  ) external view override {}

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
