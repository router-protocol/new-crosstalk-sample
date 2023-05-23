// SPDX-License-Identifier: Unlicensed
pragma solidity >=0.8.0 <0.9.0;

import "@routerprotocol/evm-gateway-contracts/contracts/IDapp.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/IGateway.sol";
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";

/// @title XERC721
/// @author Yashika Goyal
/// @notice A cross-chain ERC-721 smart contract to demonstrate how one can create
/// cross-chain NFT contracts using Router CrossTalk.
contract XERC721 is ERC721, ERC721URIStorage, IDapp {
    // address of the owner
    address public owner;

    // address of the gateway contract
    IGateway public gatewayContract;

    //  chain id => address of our contract in string
    mapping(string => string) public ourContractOnChains;

    // transfer params struct where we specify which NFT should be transferred to
    // the destination chain and to which address
    struct TransferParams {
        uint256 nftId;
        string recipient;
        string tokenURI;
    }

    constructor(
        address payable gatewayAddress,
        string memory feePayerAddress
    ) ERC721("ERC721", "ERC721") {
        gatewayContract = IGateway(gatewayAddress);
        owner = msg.sender;

        gatewayContract.setDappMetadata(feePayerAddress);
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "only owner");
        _;
    }

    /// @notice function to set the fee payer address on Router Chain.
    /// @param feePayerAddress address of the fee payer on Router Chain.
    function setDappMetadata(string memory feePayerAddress) external onlyOwner {
        gatewayContract.setDappMetadata(feePayerAddress);
    }

    /// @notice function to set the Router Gateway Contract.
    /// @param gateway address of the gateway contract.
    function setGateway(address gateway) external onlyOwner {
        gatewayContract = IGateway(gateway);
    }

    function mint(
        address account,
        uint256 nftId,
        string memory tokenUri
    ) external onlyOwner {
        _mint(account, nftId);
        _setTokenURI(nftId, tokenUri);
    }

    /// @notice function to set the address of our ERC20 contracts on different chains.
    /// This will help in access control when a cross-chain request is received.
    /// @param chainId chain Id of the destination chain in string.
    /// @param contractAddress address of the ERC20 contract on the destination chain.
    function setContractOnChain(
        string calldata chainId,
        string calldata contractAddress
    ) external onlyOwner {
        ourContractOnChains[chainId] = contractAddress;
    }

    function _burn(
        uint256 tokenId
    ) internal override(ERC721, ERC721URIStorage) {
        super._burn(tokenId);
    }

    function tokenURI(
        uint256 tokenId
    ) public view override(ERC721, ERC721URIStorage) returns (string memory) {
        return super.tokenURI(tokenId);
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

    /// @notice function to generate a cross-chain NFT transfer request.
    /// @param destChainId chain ID of the destination chain in string.
    /// @param transferParams transfer params struct.
    /// @param requestMetadata abi-encoded metadata according to source and destination chains
    function transferCrossChain(
        string calldata destChainId,
        TransferParams calldata transferParams,
        bytes calldata requestMetadata
    ) public payable {
        require(
            keccak256(abi.encodePacked(ourContractOnChains[destChainId])) !=
                keccak256(abi.encodePacked("")),
            "contract on dest not set"
        );

        require(
            _ownerOf(transferParams.nftId) == msg.sender,
            "caller is not the owner"
        );

        // burning the NFT from the address of the user calling _burn function
        _burn(transferParams.nftId);

        // sending the transfer params struct to the destination chain as payload.
        bytes memory packet = abi.encode(transferParams);
        bytes memory requestPacket = abi.encode(
            ourContractOnChains[destChainId],
            packet
        );

        gatewayContract.iSend{value: msg.value}(
            1,
            0,
            string(""),
            destChainId,
            requestMetadata,
            requestPacket
        );
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
        // decoding our payload
        TransferParams memory transferParams = abi.decode(
            packet,
            (TransferParams)
        );
        _mint(toAddress(transferParams.recipient), transferParams.nftId);

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

    /// @notice Function to convert string to address
    /// @param _str bytes to be converted
    /// @return addr address pertaining to the bytes
    function toAddress(string memory _str) public pure returns (address) {
        bytes memory addressBytes = bytes(_str);
        require(addressBytes.length == 42, "Invalid address length");
        uint8 b;
        uint160 result = 0;
        for (uint i = 2; i < addressBytes.length; i++) {
            b = uint8(addressBytes[i]);
            if (b >= 48 && b <= 57) {
                result = result * 16 + (b - 48);
            } else if (b >= 65 && b <= 70) {
                result = result * 16 + (b - 55);
            } else if (b >= 97 && b <= 102) {
                result = result * 16 + (b - 87);
            } else {
                revert("Invalid address string");
            }
        }
        return address(result);
    }
}
