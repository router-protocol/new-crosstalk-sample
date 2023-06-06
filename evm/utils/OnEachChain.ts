import { ethers } from "ethers";
import { getChainInfo } from "./chain";
import { updateChainDeploymentInfo } from "./utils";

export type DeploymentArg = {
    chainList: string[];
    contractlist: string[];
    feePayer: string | null | undefined;
};

export type Args = {
    chainList: string[];
    contractList: string[];
};

export async function deployOnEachChains(
    onSingleChain: Function,
    deploymentArg: DeploymentArg,
    signer: ethers.Signer
) {
    const { chainList, feePayer } = deploymentArg;
    console.log(`Deployment started...`);
    for (const chain of chainList) {
        const { rpc, chainName, key, chainId } = getChainInfo(chain);
        console.log(`Deploying on ${chainName}...`);
        const provider = new ethers.providers.JsonRpcProvider(rpc, {
            name: key,
            chainId: parseInt(chainId),
        });

        const deploymentData = await onSingleChain(
            signer.connect(provider),
            deploymentArg,
            getChainInfo(chain)
        );

        await updateChainDeploymentInfo(chain, deploymentData);
        console.log(`Deployment on ${chainName} completed!`);
    }
    console.log("Deployment Completed!!");
}

export async function verifyOnEachChains(
    onSingleChain: Function,
    chainList: string[],
    signer: ethers.Signer
) {
    for (const chain of chainList) {
        const { rpc, chainName, key, chainId } = getChainInfo(chain);
        console.log(`Verifying on ${chainName}...`);
        const provider = new ethers.providers.JsonRpcProvider(rpc, {
            chainId: parseInt(chainId),
            name: key,
        });
        await onSingleChain(provider, getChainInfo(chain));
    }
}

export async function enrollForEachChains(
    onSingleChain: Function,
    args: Args,
    signer: ethers.Signer,
    addOn: string[]
) {
    for (const chain of args.chainList) {
        const { rpc, chainName, key, chainId } = getChainInfo(chain);

        console.log(`Enrolling for ${chainName}: Started!!`);
        const provider = new ethers.providers.JsonRpcProvider(rpc, {
            name: key,
            chainId: parseInt(chainId),
        });

        await onSingleChain(
            signer.connect(provider),
            getChainInfo(chain),
            args,
            addOn
        );

        console.log(`Enrolling For ${chainName} Completed!!`);
        console.log();
    }
}