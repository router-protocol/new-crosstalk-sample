import { task, types } from "hardhat/config";
import { TaskArguments } from "hardhat/types";

task("TASK_DEPLOY_XERC20").setAction(async function (
  _taskArguments: TaskArguments,
  hre
) {
  const network = await hre.ethers.provider.getNetwork();
  const chainId = network.chainId;

  const deployments = require("../../deployment/deployments.json");

  const gatewayContract = deployments[chainId].gatewayContract;
  const feePayerAddress = deployments[chainId].feePayerAddress;

  const deployContract = "XERC20";

  console.log("Contract Deployment Started ");
  const Erc20 = await hre.ethers.getContractFactory("XERC20");
  const erc20 = await Erc20.deploy(
    "XERC20",
    "XERC20",
    gatewayContract,
    feePayerAddress
  );
  await erc20.deployed();

  console.log(deployContract + " Contract deployed to: ", erc20.address);
  console.log("Contract Deployment Ended");

  await hre.run("TASK_STORE_DEPLOYMENTS", {
    contractName: deployContract,
    contractAddress: erc20.address,
    chainID: chainId.toString(),
  });
  return null;
});
