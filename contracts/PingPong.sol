//SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "@routerprotocol/evm-gateway-contracts/contracts/ICrossTalkApplication.sol";
import "@routerprotocol/router-crosstalk-utils/contracts/CrossTalkUtils.sol";

/// @title PingPong
/// @author Shivam Agrawal
/// @notice This is a cross-chain ping pong smart contract to demonstrate how one can
/// utilise Router CrossTalk for cross-chain transactions.
contract PingPong is ICrossTalkApplication {
    uint64 public currentRequestId;

    // srcChainType + srcChainId + requestId => pingFromSource
    mapping(uint64 => mapping(string => mapping(uint64 => string)))
        public pingFromSource;
    // requestId => ackMessage
    mapping(uint64 => string) public ackFromDestination;

    // instance of the Router's gateway contract
    IGateway public gatewayContract;

    // gas limit required to handle the cross-chain request on the destination chain.
    uint64 public destGasLimit;

    // gas limit required to handle the acknowledgement received on the source
    // chain back from the destination chain.
    uint64 public ackGasLimit;

    // custom error so that we can emit a custom error message
    error CustomError(string message);

    // event we will emit while sending a ping to destination chain
    event PingFromSource(
        uint64 indexed srcChainType,
        string indexed srcChainId,
        uint64 indexed requestId,
        string message
    );
    event NewPing(uint64 indexed requestId);

    // events we will emit while handling acknowledgement
    event ExecutionStatus(uint64 indexed eventIdentifier, bool isSuccess);
    event AckFromDestination(uint64 indexed requestId, string ackMessage);

    constructor(
        address payable gatewayAddress,
        uint64 _destGasLimit,
        uint64 _ackGasLimit
    ) {
        gatewayContract = IGateway(gatewayAddress);
        destGasLimit = _destGasLimit;
        ackGasLimit = _ackGasLimit;
    }

    /// @notice function to generate a cross-chain request to ping a destination chain contract.
    /// @param chainType chain type of the destination chain.
    /// @param chainId chain ID of the destination chain in string.
    /// @param destGasPrice gas price of the destination chain.
    /// @param ackGasPrice gas price of the source chain.
    /// @param destinationContractAddress contract address of the contract that will handle this
    /// request on the destination chain(in bytes format).
    /// @param str string we will be sending as greeting to the destination chain.
    /// @param expiryDurationInSeconds expiry duration of the request in seconds. After this time,
    /// if the request has not already been executed, it will fail on the destination chain.
    /// If you don't want to provide any expiry duration, send type(uint64).max in its place.
    function pingDestination(
        uint64 chainType,
        string memory chainId,
        uint64 destGasPrice,
        uint64 ackGasPrice,
        address destinationContractAddress,
        string memory str,
        uint64 expiryDurationInSeconds
    ) public payable {
        currentRequestId++;
        // creating the payload to be sent to the destination chain
        bytes memory payload = abi.encode(currentRequestId, str);

        Utils.DestinationChainParams memory destChainParams = Utils
            .DestinationChainParams(
                destGasLimit,
                destGasPrice,
                chainType,
                chainId,
                "0x" // asmAddress
            );

        Utils.AckGasParams memory ackGasParams = Utils.AckGasParams(
            ackGasLimit,
            ackGasPrice
        );

        Utils.RequestArgs memory requestArgs = Utils.RequestArgs(
            uint64(block.timestamp) + expiryDurationInSeconds,
            false,
            Utils.FeePayer.APP
        );

        CrossTalkUtils.singleRequestWithAcknowledgement(
            address(gatewayContract),
            requestArgs,
            Utils.AckType.ACK_ON_SUCCESS,
            ackGasParams,
            destChainParams,
            CrossTalkUtils.toBytes(destinationContractAddress),
            payload
        );

        emit NewPing(currentRequestId);
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
        require(msg.sender == address(gatewayContract));

        (uint64 requestId, string memory sampleStr) = abi.decode(
            payload,
            (uint64, string)
        );

        if (
            keccak256(abi.encodePacked(sampleStr)) ==
            keccak256(abi.encodePacked(""))
        ) {
            revert CustomError("String should not be empty");
        }

        pingFromSource[srcChainType][srcChainId][requestId] = sampleStr;

        emit PingFromSource(srcChainType, srcChainId, requestId, sampleStr);

        return abi.encode(requestId, sampleStr);
    }

    /// @notice function to handle the acknowledgement received from the destination chain
    /// back on the source chain.
    /// @param eventIdentifier event nonce which is received when we create a cross-chain request
    /// We can use it to keep a mapping of which nonces have been executed and which did not.
    /// @param execFlags an array of boolean values suggesting whether the calls were successfully
    /// executed on the destination chain.
    /// @param execData an array of bytes returning the data returned from the handleRequestFromSource
    /// function of the destination chain.
    function handleCrossTalkAck(
        uint64 eventIdentifier,
        bool[] memory execFlags,
        bytes[] memory execData
    ) external override {
        bytes memory _execData = abi.decode(execData[0], (bytes));

        (uint64 requestId, string memory ackMessage) = abi.decode(
            _execData,
            (uint64, string)
        );

        ackFromDestination[requestId] = ackMessage;

        emit ExecutionStatus(eventIdentifier, execFlags[0]);
        emit AckFromDestination(requestId, ackMessage);
    }
}
