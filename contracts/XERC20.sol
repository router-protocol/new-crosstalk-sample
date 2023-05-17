// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "@routerprotocol/evm-gateway-contracts/contracts/IDapp.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/IGateway.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/Utils.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title CrossChain ERC-20
/// @author Yashika Goyal
/// @notice This is a cross-chain ERC-20 smart contract to demonstrate how one can
/// utilise Router CrossTalk for making cross-chain tokens
contract XERC20 is ERC20, IDapp {
  // address of the owner
  address public owner;

  // address of the gateway contract
  IGateway public gatewayContract;

  // gas limit required to handle cross-chain request on the destination chain
  uint64 public _destGasLimit;

  // chain id => address of our contract in string format
  mapping(string => string) public ourContractOnChains;

  constructor(
    string memory _name,
    string memory _symbol,
    address payable gatewayAddress,
    string memory feePayerAddress
  ) ERC20(_name, _symbol) {
    gatewayContract = IGateway(gatewayAddress);
    owner = msg.sender;

    //minting 20 tokens to deployer initially for testing
    _mint(msg.sender, 20);

    gatewayContract.setDappMetadata(feePayerAddress);
  }

  /// @notice function to set the fee payer address on Router Chain.
  /// @param feePayerAddress address of the fee payer on Router Chain.
  function setDappMetadata(string memory feePayerAddress) external {
    require(msg.sender == owner, "only owner");
    gatewayContract.setDappMetadata(feePayerAddress);
  }

  /// @notice function to set the Router Gateway Contract.
  /// @param gateway address of the gateway contract.
  function setGateway(address gateway) external {
    require(msg.sender == owner, "only owner");
    gatewayContract = IGateway(gateway);
  }

  function mint(address account, uint256 amount) external {
    require(msg.sender == owner, "only owner");
    _mint(account, amount);
  }

  /// @notice function to set the address of our ERC20 contracts on different chains.
  /// This will help in access control when a cross-chain request is received.
  /// @param chainId chain Id of the destination chain in string.
  /// @param contractAddress address of the ERC20 contract on the destination chain.
  function setContractOnChain(
    string calldata chainId,
    string calldata contractAddress
  ) external {
    require(msg.sender == owner, "only owner");
    ourContractOnChains[chainId] = contractAddress;
  }

  /// @notice function to generate a cross-chain token transfer request.
  /// @param destChainId chain ID of the destination chain in string.
  /// @param recipient address of the recipient of tokens on destination chain
  /// @param amount amount of tokens to be transferred cross-chain
  /// @param requestMetadata abi-encoded metadata according to source and destination chains
  function transferCrossChain(
    string calldata destChainId,
    bytes calldata recipient,
    uint256 amount,
    bytes calldata requestMetadata
  ) public payable {
    require(
      balanceOf(msg.sender) >= amount,
      "ERC20: Amount cannot be greater than the balance"
    );

    // burning the tokens from the address of the user calling this function
    _burn(msg.sender, amount);

    // encoding the data that we need to use on destination chain to mint the tokens there.
    bytes memory packet = abi.encode(recipient, amount);
    bytes memory requestPacket = abi.encode(
      ourContractOnChains[destChainId],
      packet
    );

    gatewayContract.iSend{ value: msg.value }(
      1,
      0,
      string(""),
      destChainId,
      requestMetadata,
      requestPacket
    );
  }

  /// @notice function to get the request metadata to be used while initiating cross-chain request
  /// @return requestMetadata abi-encoded metadata according to source and destination chains
  function getRequestMetadata(
    uint64 destGasLimit,
    uint64 destGasPrice,
    uint64 ackGasLimit,
    uint64 ackGasPrice,
    uint128 relayerFees,
    uint8 ackType,
    bool isReadCall,
    string memory asmAddress
  ) public pure returns (bytes memory) {
    bytes memory requestMetadata = abi.encodePacked(
      destGasLimit,
      destGasPrice,
      ackGasLimit,
      ackGasPrice,
      relayerFees,
      ackType,
      isReadCall,
      asmAddress
    );
    return requestMetadata;
  }

  /// @notice function to handle the cross-chain request received from some other chain.
  /// @param packet the payload sent by the source chain contract when the request was created.
  /// @param srcChainId chain ID of the source chain in string.
  function iReceive(
    string memory, //requestSender,
    bytes memory packet,
    string memory srcChainId
  ) external override returns (bytes memory) {
    require(msg.sender == address(gatewayContract), "only gateway");
    (bytes memory recipient, uint256 amount) = abi.decode(
      packet,
      (bytes, uint256)
    );
    _mint(toAddress(recipient), amount);

    return abi.encode(srcChainId);
  }

  /// @notice function to handle the acknowledgement received from the destination chain
  /// back on the source chain.
  /// @param requestIdentifier event nonce which is received when we create a cross-chain request
  /// We can use it to keep a mapping of which nonces have been executed and which did not.
  /// @param execFlag a boolean value suggesting whether the call was successfully
  /// executed on the destination chain.
  /// @param execData returning the data returned from the handleRequestFromSource
  /// function of the destination chain.
  function iAck(
    uint256 requestIdentifier,
    bool execFlag,
    bytes memory execData
  ) external override {}

  /// @notice function to convert type address into type bytes.
  function toBytes(address a) public pure returns (bytes memory b) {
    assembly {
      let m := mload(0x40)
      a := and(a, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
      mstore(add(m, 20), xor(0x140000000000000000000000000000000000000000, a))
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
}
