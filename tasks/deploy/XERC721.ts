import { task, types } from "hardhat/config";
import { TaskArguments } from "hardhat/types";

task("TASK_DEPLOY_XERC721").setAction(async function (
  _taskArguments: TaskArguments,
  hre
) {
  const network = await hre.ethers.provider.getNetwork();
  const chainId = network.chainId;

  const deployments = require("../../deployment/deployments.json");

  const gatewayContract = deployments[chainId].gatewayContract;
  const feePayerAddress = deployments[chainId].feePayerAddress;

  const deployContract = "XERC721";

  console.log("Contract Deployment Started ");
  const Erc721 = await hre.ethers.getContractFactory("XERC721");
  const erc721 = await Erc721.deploy(gatewayContract, feePayerAddress);
  await erc721.deployed();

  console.log(deployContract + " Contract deployed to: ", erc721.address);
  console.log("Contract Deployment Ended");

  await hre.run("TASK_STORE_DEPLOYMENTS", {
    contractName: deployContract,
    contractAddress: erc721.address,
    chainID: chainId.toString(),
  });
  return null;
});
