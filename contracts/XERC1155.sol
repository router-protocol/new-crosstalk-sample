// SPDX-License-Identifier: Unlicensed
pragma solidity >=0.8.0 <0.9.0;

import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";
import "@routerprotocol/router-crosstalk-utils/contracts/CrossTalkUtils.sol";
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

contract XERC1155 is ERC1155, ICrossTalkApplication {
  address public admin;
  address public gatewayContract;
  uint64 public destGasLimit;
  // chain type + chain id => address of our contract in bytes
  mapping(uint64 => mapping(string => bytes)) public ourContractOnChains;

  struct TransferParams {
    uint256[] nftIds;
    uint256[] nftAmounts;
    bytes nftData;
    bytes recipient;
  }

  constructor(
    string memory _uri,
    address payable gatewayAddress,
    uint64 _destGasLimit
  ) ERC1155(_uri) {
    gatewayContract = gatewayAddress;
    destGasLimit = _destGasLimit;
    admin = msg.sender;

    _mint(msg.sender, 1, 10, "");
  }

  function setContractOnChain(
    uint64 chainType,
    string memory chainId,
    address contractAddress
  ) external {
    require(msg.sender == admin, "only admin");
    ourContractOnChains[chainType][chainId] = toBytes(contractAddress);
  }

  function transferCrossChain(
    uint64 chainType,
    string memory chainId,
    uint64 expiryDurationInSeconds,
    uint64 destGasPrice,
    TransferParams memory transferParams
  ) public payable {
    // burning the NFTs from the address of the user calling this function
    _burnBatch(msg.sender, transferParams.nftIds, transferParams.nftAmounts);

    bytes memory payload = abi.encode(transferParams);
    uint64 expiryTimestamp = uint64(block.timestamp) + expiryDurationInSeconds;
    Utils.DestinationChainParams memory destChainParams = Utils
      .DestinationChainParams(destGasLimit, destGasPrice, chainType, chainId);

    CrossTalkUtils.singleRequestWithoutAcknowledgement(
      gatewayContract,
      expiryTimestamp,
      destChainParams,
      ourContractOnChains[chainType][chainId], // destination contract address
      payload
    );
  }

  function handleRequestFromSource(
    bytes memory srcContractAddress,
    bytes memory payload,
    string memory srcChainId,
    uint64 srcChainType
  ) external override returns (bytes memory) {
    require(msg.sender == gatewayContract);
    require(
      keccak256(srcContractAddress) ==
        keccak256(ourContractOnChains[srcChainType][srcChainId])
    );

    TransferParams memory transferParams = abi.decode(
      payload,
      (TransferParams)
    );
    _mintBatch(
      // converting the address of recipient from bytes to address
      CrossTalkUtils.toAddress(transferParams.recipient),
      transferParams.nftIds,
      transferParams.nftAmounts,
      transferParams.nftData
    );

    return abi.encode(srcChainId, srcChainType);
  }

  function handleCrossTalkAck(
    uint64, //eventIdentifier,
    bool[] memory, //execFlags,
    bytes[] memory //execData
  ) external view override {}

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
