// SPDX-License-Identifier: Unlicensed
pragma solidity >=0.8.0 <0.9.0;

import "@routerprotocol/evm-gateway-contracts/contracts/IDapp.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/IGateway.sol";
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/// @title XERC721
/// @author Yashika Goyal and Pranav Verma
/// @notice A cross-chain ERC-721 smart contract to demonstrate how one can create
/// cross-chain NFT contracts using Router CrossTalk.
contract XERC721 is ERC721,ERC721URIStorage,IDapp {
  // address of the owner
  address public owner;

 // name of the chain
  string public ChainName;

  // address of the gateway contract
  IGateway public gatewayContract;

  // chain id corresponding to chain name
  mapping(string=>string) public name;

  // set contract on source and destination chain
  mapping(string => string) public ourContractOnChains;

  // gateway address corresponding to chain name
  mapping(string=>address) public gateway;

  // transfer params struct where we specify which NFT should be transferred to
  // the destination chain and to which address
  struct TransferParams {
    uint256 nftId;
    bytes recipient;
    string uri;
  }

  struct TransferTemp{
    uint256 nftId;
    string uri;
  }



  constructor(
    string memory chainName,
    uint256 id,
    string memory uri
  ) ERC721("MyNFT", "NFT") {
    name["mumbai"]="80001";
    name["fuji"]="43113";
    gateway["mumbai"]=0x94caA85bC578C05B22BDb00E6Ae1A34878f047F7;
    gateway["fuji"]=0x94caA85bC578C05B22BDb00E6Ae1A34878f047F7;
    ChainName=chainName;
    address  gatewayAddress=gateway[chainName];
    gatewayContract = IGateway(gatewayAddress);
    owner = msg.sender;

    // setting metadata for dapp
    gatewayContract.setDappMetadata("0x1687DF89145c0530161A36f8b26733f6584FE25e");
    safeMint(msg.sender,id,uri);
    
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

function safeMint(address to, uint256 tokenId, string memory uri) public 
    {
      require(msg.sender == gateway["mumbai"] || (keccak256(abi.encodePacked(ChainName)) == keccak256(abi.encodePacked("fuji")) && msg.sender == owner), "not allowed");


        _safeMint(to, tokenId);
        _setTokenURI(tokenId, uri);
    }
     function tokenURI(uint256 tokenId)
        public
        view
        override(ERC721, ERC721URIStorage)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }
  function mint(address to, uint256 tokenId,string memory uri) external {
    require(msg.sender == owner, "only owner");
    require(keccak256(abi.encodePacked(ChainName)) == keccak256(abi.encodePacked("fuji")));
        _safeMint(to, tokenId);
        _setTokenURI(tokenId, uri);
  }

  /// @notice function to set the address of our ERC20 contracts on different chains.
  /// This will help in access control when a cross-chain request is received.

  function setContractOnChain(
    string calldata chainName,
    string calldata contractAddress
  ) external {
    require(msg.sender == owner, "only owner");
    
    ourContractOnChains[name[chainName]] = contractAddress;
  }
 function _burn(uint256 tokenId) internal override(ERC721, ERC721URIStorage) {
        super._burn(tokenId);
    }
  
  // This function sends the NFT from the source chain to the destination chain 
  function transferCrossChain(
    string calldata chainName,
   TransferTemp calldata transferTemp
   
  ) public payable {
    require(
      keccak256(bytes(ourContractOnChains[name[chainName]])) !=
        keccak256(bytes("")),
      "contract on dest not set"
    );

    require(
      _ownerOf(transferTemp.nftId) == msg.sender,
      "caller is not the owner"
    );

    TransferParams memory transferParams;
    transferParams.nftId=transferTemp.nftId;
    transferParams.recipient=toBytes(msg.sender);
    transferParams.uri=transferTemp.uri;
    // burning the NFT from the address of the user calling _burn function
    _burn(transferParams.nftId);
    string memory destChainId=name[chainName];
    // sending the transfer params struct to the destination chain as payload.
    bytes memory packet = abi.encode(transferParams);
    bytes memory requestPacket = abi.encode(
      ourContractOnChains[destChainId],
      packet
    );

    gatewayContract.iSend{ value: msg.value }(
      1,
      0,
      string(""),
      destChainId,
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

  function toBytes(address a) public pure returns (bytes memory b){
    assembly {
        let m := mload(0x40)
        a := and(a, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
        mstore(add(m, 20), xor(0x140000000000000000000000000000000000000000, a))
        mstore(0x40, add(m, 52))
        b := m
   }
}

  /// @notice function to handle the cross-chain request received from some other chain.
  /// @param requestSender address of the contract on source chain that initiated the request.
  /// @param packet the payload sent by the source chain contract when the request was created.
  /// @param srcChainId chain ID of the source chain in string.
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

    // decoding our payload
    TransferParams memory transferParams = abi.decode(packet, (TransferParams));
    string memory uri = transferParams.uri;
    safeMint(toAddress(transferParams.recipient), transferParams.nftId,uri);

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
