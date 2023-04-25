// SPDX-License-Identifier: MIT
pragma solidity ^0.8.2;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@routerprotocol/evm-gateway-contracts/contracts/IDapp.sol";

contract MockGateway is
  Initializable,
  PausableUpgradeable,
  AccessControlUpgradeable,
  UUPSUpgradeable,
  ReentrancyGuardUpgradeable
{
  struct ValsetArgs {
    // the validators in this set, represented by an Ethereum address
    address[] validators;
    // the powers of the given validators in the same order as above
    uint64[] powers;
    // the nonce of this validator set
    uint256 valsetNonce;
  }

  struct RequestPayload {
    uint256 routeAmount;
    uint256 requestIdentifier;
    uint256 requestTimestamp;
    string srcChainId;
    address routeRecipient;
    string destChainId;
    address asmAddress;
    string requestSender;
    address handlerAddress;
    bytes packet;
    bool isReadCall;
  }

  struct CrossChainAckPayload {
    uint256 requestIdentifier;
    uint256 ackRequestIdentifier;
    string destChainId;
    address requestSender;
    bytes execData;
    bool execFlag;
  }

  string public chainId;
  uint64 public eventNonce;
  uint256 public iSendDefaultFee;

  event ISendEvent(
    uint256 version,
    uint256 routeAmount,
    uint256 indexed eventNonce,
    address requestSender,
    string srcChainId,
    string destChainId,
    string routeRecipient,
    bytes requestMetadata,
    bytes requestPacket
  );

  event IReceiveEvent(
    uint256 indexed requestIdentifier,
    uint256 indexed eventNonce,
    string srcChainId,
    string destChainId,
    string relayerRouterAddress,
    string requestSender,
    bytes execData,
    bool execStatus
  );

  event IAckEvent(
    uint256 indexed eventNonce,
    uint256 indexed requestIdentifier,
    string relayerRouterAddress,
    string chainId,
    bytes data,
    bool success
  );

  function _authorizeUpgrade(
    address newImplementation
  ) internal virtual override onlyRole(DEFAULT_ADMIN_ROLE) {}

  function initialize() external initializer {
    // ACTIONS
    __AccessControl_init();
    __Pausable_init();

    // Constructor Fx
    _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);

    chainId = "1";
    eventNonce = 1;
    iSendDefaultFee = 1000000000000000;
  }

  enum FeePayer {
    APP,
    USER,
    NONE
  }

  function setDappMetadata(
    string memory feePayerAddress
  ) external payable nonReentrant returns (uint256) {
    // "fees too low" => "C03"
    eventNonce++;
    return eventNonce;
  }

  function iSend(
    uint256 version,
    uint256 routeAmount,
    string calldata routeRecipient,
    string calldata destChainId,
    bytes calldata requestMetadata,
    bytes calldata requestPacket
  ) external payable whenNotPaused returns (uint256) {
    // "fees too low" => "C03"
    //require(msg.value >= iSendDefaultFee, "C03");

    if (routeAmount > 0) {
      // "empty recipient" => "C04"
      // keccak256("") = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
      // TODO: Can we remove
      require(
        keccak256(abi.encodePacked(routeRecipient)) !=
          0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470,
        "C04"
      );

      // vault.deposit(routeAmount, msg.sender);
    }

    eventNonce++;

    emit ISendEvent(
      version,
      routeAmount,
      eventNonce,
      msg.sender,
      "1",
      destChainId,
      routeRecipient,
      requestMetadata,
      requestPacket
    );

    return eventNonce;
  }

  function iReceive(
    ValsetArgs calldata _currentValset,
    bytes[] calldata _sigs,
    RequestPayload memory requestPayload,
    string memory relayerRouterAddress
  ) external whenNotPaused nonReentrant {
    (bool execFlag, bytes memory execData) = requestPayload.handlerAddress.call(
      abi.encodeWithSelector(
        IDapp.iReceive.selector,
        requestPayload.requestSender,
        requestPayload.packet,
        requestPayload.srcChainId
      )
    );
    uint256 nonce = ++eventNonce;
    emit IReceiveEvent(
      requestPayload.requestIdentifier,
      nonce,
      requestPayload.srcChainId,
      chainId,
      relayerRouterAddress,
      requestPayload.requestSender,
      execData,
      execFlag
    );
  }

  function iAck(
    // The validators that approve the call
    ValsetArgs calldata _currentValset,
    // These are arrays of the parts of the validators signatures
    bytes[] calldata _sigs,
    CrossChainAckPayload memory crossChainAckPayload,
    string memory relayerRouterAddress
  ) external whenNotPaused {
    bool success;
    bytes memory data;
    (success, data) = crossChainAckPayload.requestSender.call(
      abi.encodeWithSelector(
        IDapp.iAck.selector,
        crossChainAckPayload.requestIdentifier,
        crossChainAckPayload.execFlag,
        crossChainAckPayload.execData
      )
    );
    uint256 nonce = ++eventNonce;
    emit IAckEvent(
      nonce,
      crossChainAckPayload.requestIdentifier,
      relayerRouterAddress,
      chainId,
      data,
      success
    );
  }

  function toBytes(address a) public pure returns (bytes memory b) {
    assembly {
      let m := mload(0x40)
      a := and(a, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF)
      mstore(add(m, 20), xor(0x140000000000000000000000000000000000000000, a))
      mstore(0x40, add(m, 52))
      b := m
    }
  }
}
