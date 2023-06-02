/* eslint-disable @typescript-eslint/no-unused-vars */
import chai, { expect } from "chai";
import { Contract } from "ethers";
import { solidity } from "ethereum-waffle";
import { ethers } from "hardhat";
import { SignerWithAddress } from "hardhat-deploy-ethers/signers";

chai.use(solidity);

describe("Ping-Pong", function () {
  let gateway: Contract;
  let pingPong: Contract;
  let owner: string;
  let otherAccount: string;

  before(async () => {
    [owner, otherAccount] = await ethers.getSigners();

    const Gateway = await ethers.getContractFactory("MockGateway");
    gateway = await Gateway.deploy();
    await gateway.initialize();
    console.log("Gateway deployed to:", gateway.address);
    const PingPong = await ethers.getContractFactory("PingPong");
    pingPong = await PingPong.connect(owner).deploy(
      gateway.address,
      "router1z6ralzg5tsznq9s6xmutyeen7evylcj7harabc"
    );
    console.log("Pingpong deployed to:", pingPong.address);
  });

  it("Should set dapp metadata if called by owner", async function () {
    await pingPong.setDappMetadata(
      "router1z6ralzg5tsznq9s6xmutyeen7evylcj7hjchjw"
    );
    console.log("metadata set");
  });

  it("Should NOT set dapp metadata if not called by owner", async function () {
    await expect(
      pingPong
        .connect(otherAccount)
        .setDappMetadata("router1z6ralzg5tsznq9s6xmutyeen7evylcj7hjchjw")
    ).to.be.revertedWith("only owner");
  });

  it("Should set gateway if called by owner", async function () {
    await pingPong.setGateway(gateway.address);
    expect(await pingPong.gatewayContract()).to.be.equal(gateway.address);
  });

  it("Should NOT set dapp metadata if not called by owner", async function () {
    await expect(
      pingPong.connect(otherAccount).setGateway(gateway.address)
    ).to.be.revertedWith("only owner");
  });

  it("Should send iPing", async function () {
    const metadata = await pingPong.getRequestMetadata(
      300000,
      30000000000,
      300000,
      30000000000,
      10000000000,
      1,
      false,
      "0x"
    );
    await pingPong.iPing("1", pingPong.address, "hello ping", metadata);
    const Logs1 = await pingPong.queryFilter("NewPing");
    const Logs2 = await gateway.queryFilter("ISendEvent");

    console.log("LOGS1:", Logs1[0].args);
    console.log("LOGS2:", Logs2[0].args);
    // await gateway.iReceive(
    //   [[otherAccount], [10], 1],
    //   ["0x23"],
    //   [
    //     0,
    //     1,
    //     10000,
    //     "1",
    //     "0x",
    //     "1",
    //     "0x",
    //     pingPong.address,
    //     pingPong.address,
    //     Logs[0].args.requestPacket,
    //     false,
    //   ],
    //   ""
    // );
  });
});
