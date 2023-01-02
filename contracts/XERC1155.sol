pragma solidity >=0.8.0 <0.9.0;

import "evm-gateway-contract/contracts/IGateway.sol";
import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

contract XERC1155 is ERC1155, ICrossTalkApplication {
  address public admin;
  IGateway public gatewayContract;
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
    string memory uri,
    address payable gatewayAddress,
    uint64 _destGasLimit
  ) ERC1155(uri) {
    gatewayContract = IGateway(gatewayAddress);
    destGasLimit = _destGasLimit;
    admin = msg.sender;
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

    bytes[] memory addresses = new bytes[](1);
    // fetching the address of NFT contract address on the destination chain
    addresses[0] = ourContractOnChains[chainType][chainId];

    bytes[] memory payloads = new bytes[](1);
    payloads[0] = payload;

    sendCrossChain(
      addresses,
      payloads,
      Utils.DestinationChainParams(
        destGasLimit,
        destGasPrice,
        chainType,
        chainId
      ),
      expiryTimestamp
    );
  }

  function sendCrossChain(
    bytes[] memory addresses,
    bytes[] memory payloads,
    Utils.DestinationChainParams memory destChainParams,
    uint64 expiryTimestamp
  ) internal {
    gatewayContract.requestToDest(
      expiryTimestamp,
      false,
      Utils.AckType.NO_ACK,
      Utils.AckGasParams(0, 0),
      destChainParams,
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
      toAddress(transferParams.recipient),
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

  function toAddress(
    bytes memory _bytes
  ) public pure returns (address contractAddress) {
    bytes20 srcTokenAddress;
    assembly {
      srcTokenAddress := mload(add(_bytes, 0x20))
    }
    contractAddress = address(srcTokenAddress);
  }
}
