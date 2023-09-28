import { task } from "hardhat/config";
import { TaskArguments } from "hardhat/types";

task("TASK_DEPLOY_PINGPONG")
.addParam("gateway", "Gateway Contract Address")
.addParam("feepayer", "Fee Payer Address")
.setAction(async function (
  _taskArguments: TaskArguments,
  hre
) {
  const chainId = await hre.network.config.chainId;
  if (chainId == undefined) {
    return;
  }

  const gatewayContract = _taskArguments.gateway;
  const feePayerAddress = _taskArguments.feepayer;
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
