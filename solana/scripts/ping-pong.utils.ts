import { Provider } from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import { PingPong } from "../target/types/ping_pong";
import {
  decodeRequestPacket,
  getEventByName,
  getGatewayPdas,
  getPingPongPdas,
  getRandomInt,
  getRequestMetadataEncodePacked,
  PACKET_SEED_PREFIX,
} from "./utils";
import * as ethers from "ethers";

export async function initialize(
  provider: Provider,
  pingPongInstance: anchor.Program<PingPong>,
  signer: anchor.web3.Keypair,
  args: string[]
) {
  const [pingPongAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("ping_pong")],
    pingPongInstance.programId
  );
  const instruction = await pingPongInstance.methods
    .initialize(
      args[0],
      new anchor.web3.PublicKey(args[1]),
      new anchor.BN(args.length > 2 ? args[2] : 0),
      new anchor.web3.PublicKey(args.length > 3 ? args[3] : signer.publicKey)
    )
    .accounts({
      pingPongAccount,
      signer: signer.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .instruction();

  const sig = await anchor.web3.sendAndConfirmTransaction(
    provider.connection,
    new anchor.web3.Transaction().add(instruction),
    [signer],
    { "commitment": "confirmed" }
  );
  console.log("Initialize] Sig: ", sig);
}

export async function setDappMetadata(
  provider: Provider,
  pingPongInstance: anchor.Program<PingPong>,
  signer: anchor.web3.Keypair,
  args: string[]
) {
  const gatewayProgramId = new anchor.web3.PublicKey(args[0]);
  const pdas = getGatewayPdas(gatewayProgramId);
  const [pingPongAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("ping_pong")],
    pingPongInstance.programId
  );
  const [dappAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("dapp_account"),
      pingPongAccount.toBuffer(),
    ],
    gatewayProgramId
  );
  const instruction = await pingPongInstance.methods
    .setDappMetadata(args[1])
    .accounts({
      pingPongAccount,
      signer: signer.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      gatewayProgram: gatewayProgramId,
      gatewayAccount: pdas.gatewayAccount.account,
      gatewayDappAccount: dappAccount,
      gatewayEventAuthority: pdas.eventAuthority.account,
    })
    .instruction();

  const sig = await anchor.web3.sendAndConfirmTransaction(
    provider.connection,
    new anchor.web3.Transaction().add(instruction),
    [signer],
    { "commitment": "confirmed" }
  );
  console.log("SetMetadata] Sig: ", sig);
}

export async function ping(
  provider: Provider,
  pingPongInstance: anchor.Program<PingPong>,
  signer: anchor.web3.Keypair,
  args: string[]
) {
  const gatewayProgramId = new anchor.web3.PublicKey(args[0]);
  const dstChainId = args[1];
  const dstContract = args[2];

  const pdas = getGatewayPdas(gatewayProgramId);
  const pingPongPdas = getPingPongPdas(
    pingPongInstance.programId,
    gatewayProgramId
  );
  const requestMetadata = ethers.getBytes(
    getRequestMetadataEncodePacked(500000, 0, 100000, 0, 0, 3, false, "")
  );
  const randomBytes = ethers.randomBytes(getRandomInt(6, 16));
  const packetSeed = Buffer.from(randomBytes);
  const [requestPacket] = anchor.web3.PublicKey.findProgramAddressSync(
    [PACKET_SEED_PREFIX, packetSeed],
    gatewayProgramId
  );

  const instruction = await pingPongInstance.methods
    .iPing(
      packetSeed,
      new anchor.BN(0), // version
      new anchor.BN(0), //route_amount
      "", // route_recipient
      dstChainId,
      Buffer.from(ethers.toUtf8Bytes(dstContract)),
      Buffer.from(requestMetadata)
    )
    .accounts({
      pingPongAccount: pingPongPdas.pingPongAccount.account,
      gatewayAccount: pdas.gatewayAccount.account,
      requestPacket,
      gatewayDappAccount: pingPongPdas.dappAccount.account,
      signer: signer.publicKey,
      signerAssociateAccount: null,
      mint: null,
      associatedTokenProgram: null,
      tokenProgram: null,
      gatewayEventAuthority: pdas.eventAuthority.account,
      gatewayProgram: gatewayProgramId,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .instruction();

  const sig = await anchor.web3.sendAndConfirmTransaction(
    provider.connection,
    new anchor.web3.Transaction().add(instruction),
    [signer],
    { "commitment": "confirmed" }
  );
  console.log("SetMetadata] Sig: ", sig);
  console.log(
    "NewPingEvent: ",
    await getEventByName(provider, sig, pingPongInstance, "NewPing")
  );

  console.log(await decodeRequestPacket(provider, requestPacket));
}
