// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.0 <0.9.0;

import "@routerprotocol/evm-gateway-contracts/contracts/ICrossTalkApplication.sol";
import "@routerprotocol/router-crosstalk-utils/contracts/CrossTalkUtils.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract XERC20 is ERC20, ICrossTalkApplication {
    // address of the owner
    address public owner;

    // address of the gateway contract
    IGateway public gatewayContract;

    // gas limit required to handle cross-chain request on the destination chain
    uint64 public destGasLimit;

    // chain type + chain id => address of our contract in bytes
    mapping(uint64 => mapping(string => bytes)) public ourContractOnChains;

    constructor(
        string memory _name,
        string memory _symbol,
        address payable gatewayAddress,
        uint64 _destGasLimit,
        string memory feePayerAddress
    ) ERC20(_name, _symbol) {
        gatewayContract = IGateway(gatewayAddress);
        destGasLimit = _destGasLimit;
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
    /// @param chainType chain type of the destination chain.
    /// @param chainId chain Id of the destination chain in string.
    /// @param contractAddress address of the ERC20 contract on the destination chain.
    function setContractOnChain(
        uint64 chainType,
        string memory chainId,
        address contractAddress
    ) external {
        require(msg.sender == owner, "only owner");
        ourContractOnChains[chainType][chainId] = toBytes(contractAddress);
    }

    /// @notice function to generate a cross-chain token transfer request.
    /// @param chainType chain type of the destination chain.
    /// @param chainId chain ID of the destination chain in string.
    /// @param expiryDurationInSeconds expiry duration of the request in seconds. After this time,
    /// if the request has not already been executed, it will fail on the destination chain.
    /// If you don't want to provide any expiry duration, send type(uint64).max in its place.
    /// @param destGasPrice gas price of the destination chain.
    /// @param recipient address of the recipient of tokens on destination chain
    /// @param amount amount of tokens to be transferred cross-chain
    function transferCrossChain(
        uint64 chainType,
        string memory chainId,
        uint64 expiryDurationInSeconds,
        uint64 destGasPrice,
        bytes memory recipient,
        uint256 amount
    ) public payable {
        require(
            balanceOf(msg.sender) >= amount,
            "ERC20: Amount cannot be greater than the balance"
        );

        // burning the tokens from the address of the user calling this function
        _burn(msg.sender, amount);

        // encoding the data that we need to use on destination chain to mint the tokens there.
        bytes memory payload = abi.encode(recipient, amount);

        // timestamp when the call expires. If this time passes by, the call will fail on the destination chain.
        // If you don't want to add an expiry timestamp, set it to zero.
        uint64 expiryTimestamp = uint64(block.timestamp) +
            expiryDurationInSeconds;

        // Destination chain params is a struct that consists of gas limit and gas price for destination chain,
        // chain type of the destination chain and chain id of destination chain.
        Utils.DestinationChainParams memory destChainParams = Utils
            .DestinationChainParams(
                destGasLimit,
                destGasPrice,
                chainType,
                chainId,
                "" // asmAddress
            );

        /// requestArgs consists of expiryTimestamp, isAtomicCalls boolean and feePayerEnum.
        Utils.RequestArgs memory requestArgs = Utils.RequestArgs(
            expiryTimestamp,
            false
        );

        // This is the function to send a single request without acknowledgement to the destination chain.
        // You will be able to send a single request to a single contract on the destination chain and
        // you don't need the acknowledgement back on the source chain.
        CrossTalkUtils.singleRequestWithoutAcknowledgement(
            address(gatewayContract),
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
        // Checks if the contract that triggered this function is address of our gateway contract
        require(msg.sender == address(gatewayContract));
        require(
            keccak256(srcContractAddress) ==
                keccak256(ourContractOnChains[srcChainType][srcChainId])
        );
        // decoding the data that we encoded to be used for minting the tokens on destination chain.
        (bytes memory recipient, uint256 amount) = abi.decode(
            payload,
            (bytes, uint256)
        );
        // mints the tokens to recipient on destination chain
        _mint(
            // converting the address of recipient from bytes to address
            CrossTalkUtils.toAddress(recipient),
            amount
        );

        return abi.encode(srcChainId, srcChainType);
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

    /// @notice function to convert type address into type bytes.
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
}
