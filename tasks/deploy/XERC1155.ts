import { task, types } from "hardhat/config";
import { TaskArguments } from "hardhat/types";

task("TASK_DEPLOY_XERC1155").setAction(async function (
  _taskArguments: TaskArguments,
  hre
) {
  const network = await hre.ethers.provider.getNetwork();
  const chainId = network.chainId;

  const deployments = require("../../deployment/deployments.json");

  const gatewayContract = deployments[chainId].gatewayContract;
  const feePayerAddress = deployments[chainId].feePayerAddress;
  const deployContract = "XERC1155";

  console.log("Contract Deployment Started ");
  const Erc1155 = await hre.ethers.getContractFactory("XERC1155");
  const erc1155 = await Erc1155.deploy("uri", gatewayContract, feePayerAddress);
  await erc1155.deployed();

  console.log(deployContract + " Contract deployed to: ", erc1155.address);
  console.log("Contract Deployment Ended");

  await hre.run("TASK_STORE_DEPLOYMENTS", {
    contractName: deployContract,
    contractAddress: erc1155.address,
    chainID: chainId.toString(),
  });
  return null;
});
