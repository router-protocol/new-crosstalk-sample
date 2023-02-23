// SPDX-License-Identifier: Unlicensed
pragma solidity >=0.8.0 <0.9.0;

import "evm-gateway-contract/contracts/ICrossTalkApplication.sol";
import "evm-gateway-contract/contracts/Utils.sol";
import "@routerprotocol/router-crosstalk-utils/contracts/CrossTalkUtils.sol";
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

/// @title XERC1155
/// @notice A cross-chain ERC-1155 smart contract to demonstrate how one can create
/// cross-chain NFT contracts using Router CrossTalk.
contract XERC1155 is ERC1155, ICrossTalkApplication {
    // address of the admin
    address public admin;

    // address of the gateway contract
    address public gatewayContract;

    // gas limit required to handle cross-chain request on the destination chain
    uint64 public destGasLimit;

    // chain type + chain id => address of our contract in bytes
    mapping(uint64 => mapping(string => bytes)) public ourContractOnChains;

    // transfer params struct where we specify which NFTs should be transferred to
    // the destination chain and to which address
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

        // minting ourselves some NFTs so that we can test out the contracts
        _mint(msg.sender, 1, 10, "");
    }

    /// @notice function to set the address of our NFT contracts on different chains.
    /// This will help in access control when a cross-chain request is received.
    /// @param chainType chain type of the destination chain.
    /// @param chainId chain Id of the destination chain in string.
    /// @param contractAddress address of the NFT contract on the destination chain.
    function setContractOnChain(
        uint64 chainType,
        string memory chainId,
        address contractAddress
    ) external {
        require(msg.sender == admin, "only admin");
        ourContractOnChains[chainType][chainId] = CrossTalkUtils.toBytes(
            contractAddress
        );
    }

    /// @notice function to generate a cross-chain NFT transfer request.
    /// @param chainType chain type of the destination chain.
    /// @param chainId chain ID of the destination chain in string.
    /// @param expiryDurationInSeconds expiry duration of the request in seconds. After this time,
    /// if the request has not already been executed, it will fail on the destination chain.
    /// If you don't want to provide any expiry duration, send type(uint64).max in its place.
    /// @param destGasPrice gas price of the destination chain.
    /// @param transferParams transfer params struct.
    function transferCrossChain(
        uint64 chainType,
        string memory chainId,
        uint64 expiryDurationInSeconds,
        uint64 destGasPrice,
        TransferParams memory transferParams
    ) public payable {
        require(
            keccak256(ourContractOnChains[chainType][chainId]) !=
                keccak256(CrossTalkUtils.toBytes(address(0))),
            "contract on dest not set"
        );

        // burning the NFTs from the address of the user calling _burnBatch function
        _burnBatch(
            msg.sender,
            transferParams.nftIds,
            transferParams.nftAmounts
        );

        // sending the transfer params struct to the destination chain as payload.
        bytes memory payload = abi.encode(transferParams);

        // creating the expiry timestamp
        uint64 expiryTimestamp = uint64(block.timestamp) +
            expiryDurationInSeconds;
        Utils.DestinationChainParams memory destChainParams = Utils
            .DestinationChainParams(
                destGasLimit,
                destGasPrice,
                chainType,
                chainId
            );

        Utils.RequestArgs memory requestArgs = Utils.RequestArgs(
            expiryTimestamp,
            false,
            Utils.FeePayer.APP
        );

        // creating a cross-chain communication request to the destination chain.
        CrossTalkUtils.singleRequestWithoutAcknowledgement(
            gatewayContract,
            requestArgs,
            destChainParams,
            ourContractOnChains[chainType][chainId], // destination contract address
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
        require(msg.sender == gatewayContract, "only gateway");
        // ensuring that our NFT contract initiated this request from the source chain
        require(
            keccak256(srcContractAddress) ==
                keccak256(ourContractOnChains[srcChainType][srcChainId]),
            "only our contract on source chain"
        );

        // decoding our payload
        TransferParams memory transferParams = abi.decode(
            payload,
            (TransferParams)
        );

        // minting the recipient the respective token ids and amounts
        _mintBatch(
            // converting the address of recipient from bytes to address
            CrossTalkUtils.toAddress(transferParams.recipient),
            transferParams.nftIds,
            transferParams.nftAmounts,
            transferParams.nftData
        );

        // since we don't want to return any data, we will just return empty string
        return "";
    }

    /// @notice function to handle the acknowledgement received from the destination chain
    /// back on the source chain.
    /// @param eventIdentifier event nonce which is received when we create a cross-chain request
    /// We can use it to keep a mapping of which nonces have been executed and which did not.
    /// @param execFlags an array of boolean values suggesting whether the calls were successfully
    /// executed on the destination chain.
    /// @param execData an array of bytes returning the data returned from the handleRequestFromSource
    /// function of the destination chain.
    /// Since we don't want to handle the acknowledgement, we will leave it as empty function.
    function handleCrossTalkAck(
        uint64 eventIdentifier,
        bool[] memory execFlags,
        bytes[] memory execData
    ) external view override {}
}
