// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "@routerprotocol/evm-gateway-contracts/contracts/IGateway.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/ICrossTalkApplication.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/Utils.sol";
import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

/// @title XERC1155
/// @author Yashika Goyal
/// @notice A cross-chain ERC-1155 smart contract to demonstrate how one can create
/// cross-chain NFT contracts using Router CrossTalk.
contract XERC1155 is ERC1155, ICrossTalkApplication {
    // address of the admin
    address public admin;

    // address of the gateway contract
    IGateway public gatewayContract;

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
        gatewayContract = IGateway(gatewayAddress);
        destGasLimit = _destGasLimit;
        admin = msg.sender;

        // Mint 10 NFTs of ID 1 to msg sender
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
        ourContractOnChains[chainType][chainId] = toBytes(contractAddress);
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
        // burning the NFTs from the address of the user calling this function
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

        // creating an array of destination contract addresses in bytes
        bytes[] memory addresses = new bytes[](1);
        // fetching the address of NFT contract address on the destination chain
        addresses[0] = ourContractOnChains[chainType][chainId];

        // creating an array of payloads to be sent to respective destination contracts
        bytes[] memory payloads = new bytes[](1);
        payloads[0] = payload;

        // sending a cross-chain request
        sendCrossChain(
            addresses,
            payloads,
            Utils.DestinationChainParams(
                destGasLimit,
                destGasPrice,
                chainType,
                chainId,
                "0x" // asmAddress
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
        Utils.RequestArgs memory requestArgs = Utils.RequestArgs(
            expiryTimestamp,
            false,
            Utils.FeePayer.APP
        );

        // Calling the requestToDest function on the Gateway contract to generate a cross-chain request
        gatewayContract.requestToDest(
            requestArgs,
            Utils.AckType.NO_ACK,
            Utils.AckGasParams(0, 0),
            destChainParams,
            Utils.ContractCalls(payloads, addresses)
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
        require(msg.sender == address(gatewayContract), "only gateway");

        // ensuring that our NFT contract initiated this request from the source chain
        require(
            keccak256(srcContractAddress) ==
                keccak256(ourContractOnChains[srcChainType][srcChainId]),
            "only our contract"
        );

        // decoding our payload
        TransferParams memory transferParams = abi.decode(
            payload,
            (TransferParams)
        );

        // minting the recipient the respective token ids and amounts
        _mintBatch(
            // converting the address of recipient from bytes to address
            toAddress(transferParams.recipient),
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

    /// @notice function to convert address to bytes
    function toBytes(address a) public pure returns (bytes memory b) {
        assembly {
            let m := mload(0x40)
            a := and(a, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
            mstore(
                add(m, 20),
                xor(0x140000000000000000000000000000000000000000, a)
            )
            mstore(0x40, add(m, 52))
            b := m
        }
    }

    /// @notice function to convert bytes to address
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
