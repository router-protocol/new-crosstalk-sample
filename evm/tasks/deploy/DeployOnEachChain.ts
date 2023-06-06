import { task } from "hardhat/config";
import { TaskArguments } from "hardhat/types";
import { ethers } from "ethers";
import { ChainType, ReturnType } from "../../utils/types";
import {
  getChainDeployment,
  getPrevDeployment,
  getSignerFromPrivateKeyOrMnemonic,
  isPreviousDeployed,
} from "../../utils/utils";
import { getChainId } from "../../utils/chain";
import { deployOnEachChains } from "../../utils/OnEachChain";
import { DeploymentArg } from "../../utils/OnEachChain";
import { log } from "console";

// Create a new provider using the custom network configuration
let hre: any;
async function deployOnSingleChain(
  signer: ethers.Signer,
  deploymentArg: DeploymentArg,
  chainInfo: ChainType
): Promise<ReturnType> {
  const hre = require("hardhat");
  const provider = signer.provider;
  if (!provider) throw new Error("signer provider is undefined");

  hre.provider = provider;
  hre.network.provider = provider;
  const { ethers } = hre;

  if (!deploymentArg.feePayer) {
    deploymentArg.feePayer = (
      await getChainDeployment(chainInfo.chainId)
    )?.feePayer;
  }

  if (!deploymentArg.feePayer) throw new Error("Feepayer is undefined");

  let deploymentData: { [name: string]: string } = {};

  for (const contract of deploymentArg.contractlist) {
    console.log(`Deploying ${contract}...`);
    // can be PingPong or XERC1155
    const Contract = await (
      await ethers.getContractFactory(contract)
    ).connect(signer);

    const contract_instance =
      contract == "PingPong"
        ? await Contract.deploy(
          chainInfo.gateway,
          1000000,
          1000000,
          deploymentArg.feePayer
        )
        : await Contract.deploy(
          "uri",
          chainInfo.gateway,
          1000000,
          deploymentArg.feePayer
        );

    await contract_instance.deployed();
    deploymentData[contract] = contract_instance.address;
    console.log(
      `Deployed ${contract} successfully with address ${contract_instance.address}`
    );
  }
  return deploymentData;
}

task("DEPLOY_ONEACH", "deploy contracts on provided chains")
  .addParam("chainlist", "Description of chainlist parameter")
  .addOptionalParam("contractlist", "pass list of contract to be deployed")
  .addOptionalParam("feepayer", "pass same feepayer for each deployment")
  .addOptionalParam(
    "pkm",
    "Description of private key or mnemonic as parameter"
  )
  .setAction(async (taskArgs: TaskArguments, hre: any) => {
    const { chainlist, pkm, feepayer, contractlist } = taskArgs;
    let signer;
    if (pkm) signer = getSignerFromPrivateKeyOrMnemonic(pkm);
    else {
      // load from env
      const morp = process.env.MNEMONIC || process.env.PRIVATE_KEY;
      if (!morp) throw new Error("Provide mnemonic or private key");
      signer = getSignerFromPrivateKeyOrMnemonic(morp);
    }
    const chainList: string[] = Array.from(
      new Set(chainlist.trim().split(" "))
    );
    const contractList: string[] = contractlist
      ? Array.from(new Set(contractlist.trim().split(" ")))
      : [];
    if (!contractList.length) contractList.push("PingPong"); // default config

    chainList.map((chain: string) => {
      if (!getChainId(chain)) throw new Error("invalid chain provided");
    });
    await deployOnEachChains(
      deployOnSingleChain,
      { chainList, feePayer: feepayer, contractlist: contractList },
      signer
    );

    let args: {
      chainlist: string;
      contractlist: string;
      pkm?: string;
      enrollwith?: string;
    } = {
      chainlist,
      contractlist,
    };
    if (pkm)
      args = {
        ...args,
        pkm,
      };

    console.log();
    const prevDeployment = await getPrevDeployment();
    if (Object.keys(prevDeployment).length) {
      let enrollwith = "";
      Object.keys(prevDeployment).map((key) => (enrollwith += `${key} `));
      args = {
        ...args,
        enrollwith,
      };
      await hre.run("ENROLLADDED_ONEACH", args);
    } else {
      await hre.run("ENROLL_ONEACH", args);
    }
  });