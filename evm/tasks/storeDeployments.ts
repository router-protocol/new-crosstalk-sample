import { task, types } from "hardhat/config";
import fs from "fs";

task("TASK_STORE_DEPLOYMENTS", "stores deployment addresses")
  .addParam<string>("contractName", "Contract Name", "", types.string)
  .addParam<string>("contractAddress", "Contract Address", "", types.string)
  .addParam<string>("chainID", "chain id", "", types.string)
  .setAction(async (taskArgs, { ethers }): Promise<null> => {
    const deployedContracts = require("../deployment/deployments.json");

    if (typeof deployedContracts[taskArgs.chainID] === "undefined") {
      deployedContracts[taskArgs.chainID] = {};
    }

    deployedContracts[taskArgs.chainID][taskArgs.contractName] =
      taskArgs.contractAddress;

    fs.writeFileSync(
      "deployment/deployments.json",
      JSON.stringify(deployedContracts)
    );

    return null;
  });
