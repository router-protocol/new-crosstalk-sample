// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

// import "@routerprotocol/evm-gateway-contracts/contracts/IGateway.sol";
// import "@routerprotocol/evm-gateway-contracts/contracts/Utils.sol";
// import "@routerprotocol/evm-gateway-contracts/contracts/IApplication.sol";

import "@routerprotocol/evm-gateway-contracts/contracts/IGateway.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/IDapp.sol";
import "hardhat/console.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

// DevNet Gateway 0xB139915AE11f6f0ACd05C8dB85E8ED1bE1c7c17d
// AlphaDevNet
contract TestDapp {
    IGateway public gatewayContract;
    IERC20 public routeToken;

    mapping(string => mapping(uint64 => string)) public greetingRecord;
    mapping(uint256 => bool) public ackRecords;

    error CustomError(string message);

    constructor(
        address payable gatewayAddress,
        string memory feePayer,
        address routeTokenAddr
    ) {
        gatewayContract = IGateway(gatewayAddress);
        gatewayContract.setDappMetadata(feePayer);
        routeToken = IERC20(routeTokenAddr);
    }

    function setDappMetadata(string memory feePayer) external {
        gatewayContract.setDappMetadata(feePayer);
    }

    function giveAllowance(
        address tokenAddress,
        address vaultAddress,
        uint256 amount
    ) external {
        uint256 currentallowance = IERC20(tokenAddress).allowance(
            address(this),
            vaultAddress
        );
        IERC20(tokenAddress).approve(vaultAddress, currentallowance + amount);
    }

    function sendIRequest(
        bytes calldata payload,
        string calldata destContractAddress,
        string calldata destChainId,
        bytes calldata requestMetadata,
        uint256 amount,
        string calldata routeRecipient
    ) external payable returns (uint256) {
        routeToken.transferFrom(msg.sender, address(this), amount);
        bytes memory requestPacket = abi.encode(destContractAddress, payload);

        uint256 nonce = iSend(
            amount,
            routeRecipient,
            destChainId,
            requestMetadata,
            requestPacket
        );

        (, string memory greeting) = abi.decode(payload, (uint64, string));
        require(
            keccak256(abi.encode(greeting)) != keccak256(abi.encode("")),
            "greeting cannot be empty"
        );

        return nonce;
    }

    function iSend(
        uint256 amount,
        string calldata routeRecipient,
        string calldata destChainId,
        bytes calldata requestMetadata,
        bytes memory requestPacket
    ) internal returns (uint256) {
        return
            gatewayContract.iSend{value: msg.value}(
                1,
                amount,
                routeRecipient,
                destChainId,
                requestMetadata,
                requestPacket
            );
    }

    function iReceive(
        string memory srcContractAddress,
        bytes memory packet,
        string memory srcChainId
    ) external returns (uint64, string memory) {
        require(msg.sender == address(gatewayContract));

        (uint64 nonce, string memory greeting) = abi.decode(
            packet,
            (uint64, string)
        );

        if (keccak256(bytes(greeting)) == keccak256(bytes("Fail Dest Req"))) {
            revert CustomError("String != Fail Dest Req");
        }

        greetingRecord[srcChainId][nonce] = greeting;
        return (nonce, greeting);
    }

    function iAck(
        uint256 eventIdentifier,
        bool execFlag,
        bytes memory execData
    ) external {
        if (execFlag) {
            (, string memory greeting) = abi.decode(execData, (uint64, string));
            if (
                keccak256(bytes(greeting)) == keccak256(bytes("Fail Ack Req"))
            ) {
                revert CustomError("String != Fail Ack Req");
            }
        }

        ackRecords[eventIdentifier] = true;
    }

    receive() external payable {}
}
