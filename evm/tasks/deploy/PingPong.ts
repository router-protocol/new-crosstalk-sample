import { task, types } from "hardhat/config";
import { TaskArguments } from "hardhat/types";

task("TASK_DEPLOY_PINGPONG").setAction(async function (
  _taskArguments: TaskArguments,
  hre
) {
  const network = await hre.ethers.provider.getNetwork();
  const chainId = network.chainId;

  const deployments = require("../../deployment/deployments.json");

  const gatewayContract = deployments[chainId].gatewayContract;
  const feePayerAddress = deployments[chainId].feePayerAddress;
  const deployContract = "PingPong";

  console.log("Contract Deployment Started ");
  const PingPong = await hre.ethers.getContractFactory("PingPong");
  const pingPong = await PingPong.deploy(gatewayContract, feePayerAddress);

  await pingPong.deployed();

  console.log(deployContract + " Contract deployed to: ", pingPong.address);
  console.log("Contract Deployment Ended");

  await hre.run("TASK_STORE_DEPLOYMENTS", {
    contractName: deployContract,
    contractAddress: pingPong.address,
    chainID: chainId.toString(),
  });
  return null;
});
