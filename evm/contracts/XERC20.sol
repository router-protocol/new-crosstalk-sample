// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "@routerprotocol/evm-gateway-contracts/contracts/IDapp.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/IGateway.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/Utils.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @title XERC20
/// @author Yashika Goyal
/// @notice This is a cross-chain ERC-20 smart contract to demonstrate how one can
/// utilise Router CrossTalk for making cross-chain tokens
contract XERC20 is ERC20, IDapp {
  // address of the owner
  address public owner;

  // name of the chain
  string public ChainName;

  // address of the gateway contract
  IGateway public gatewayContract;

  // gas limit required to handle cross-chain request on the destination chain
  uint64 public _destGasLimit;

// chain id corresponding to chain name
  mapping(string=>string) public name;

  // set contract on source and destination chain
  mapping(string => string) public ourContractOnChains;

  // gateway address corresponding to chain name
  mapping(string=>address) public gateway;

  constructor(
    string memory chainName,
    uint256 amount
  ) ERC20("My Token", "MTK") {

    name["mumbai"]="80001";
    name["fuji"]="43113";
    gateway["mumbai"]=0x94caA85bC578C05B22BDb00E6Ae1A34878f047F7;
    gateway["fuji"]=0x94caA85bC578C05B22BDb00E6Ae1A34878f047F7;
    ChainName=chainName;
    address  gatewayAddress=gateway[chainName];
    gatewayContract = IGateway(gatewayAddress);
    owner = msg.sender;
    _mint(msg.sender,amount);

    // setting metadata for dapp
   
    gatewayContract.setDappMetadata("0xdeF7d841DEFC2B6d201958d6Ba98bF6eAd884f6d");
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

  
   function setContractOnChain(
    string calldata chainName,
    string calldata contractAddress
  ) external {
    require(msg.sender == owner, "only owner");
    
    ourContractOnChains[name[chainName]] = contractAddress;
  }
 
  function transferCrossChain(
    uint256 amount,
    string memory destinationChain
  ) public payable {

    require(
      keccak256(bytes(ourContractOnChains[name[destinationChain]])) !=
        keccak256(bytes("")),
      "contract on dest not set"
    );
   

    require(
      balanceOf(msg.sender) >= amount,
      "ERC20: Amount cannot be greater than the balance"
    );

    // burning the tokens from the address of the user calling this function
    _burn(msg.sender, amount);

    // encoding the data that we need to use on destination chain to mint the tokens there.
    bytes memory packet = abi.encode(msg.sender, amount);
    bytes memory requestPacket = abi.encode(
      ourContractOnChains[name[destinationChain]],
      packet
    );

     gatewayContract.iSend{ value: msg.value }(
      1,
      0,
      string(""),
      name[destinationChain],
     hex"000000000007a12000000006fc23ac0000000000000000000000000000000000000000000000000000000000000000000000",
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
    string calldata asmAddress
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


   function iReceive(
    string memory requestSender,
    bytes memory packet,
    string memory srcChainId
  ) external override returns (bytes memory) {
    require(msg.sender == address(gatewayContract), "only gateway");
    require(
      keccak256(bytes(ourContractOnChains[srcChainId])) ==
        keccak256(bytes(requestSender))
    );


    (address recipient, uint256 amount) = abi.decode(
      packet,
      (address, uint256)
    );
    // _mint(toAddress(recipient), amount);
    
     _mint(owner, amount);

    return "";
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
  /// @param a address to be converted
  /// @return b bytes pertaining to the address
 

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
