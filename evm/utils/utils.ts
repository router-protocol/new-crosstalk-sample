import path from "path";
import fs from "fs-extra";
import { ethers } from "ethers";
import { getChainId } from "./chain";
import { JsonType, ContractInfo } from "./types";

const filePath = path.resolve(__dirname, "../deployment/deployments.json");

export async function getChainDeployment(
    chain: string
): Promise<ContractInfo | null> {
    try {
        const json: JsonType = await fs.readJson(filePath);
        const chainId = getChainId(chain);
        if (!chainId) throw new Error("Invalid chain");
        return json[chainId];
    } catch (err) {
        //@ts-ignore
        console.error(`Failed to read file: ${err.message}`);
        process.exit(1);
    }
}

export async function isPreviousDeployed(): Promise<boolean> {
    try {
        const json: JsonType = await fs.readJson(filePath);
        if (Object.keys(json).length) return true;
        return false;
    } catch (err) {
        return false;
    }
}

export async function getPrevDeployment(): Promise<JsonType> {
    try {
        return await fs.readJson(filePath);
    } catch (err) {
        return {};
    }
}

export function getSignerFromPrivateKeyOrMnemonic(str: string) {
    const privateKeyRegex = /^[0-9a-fA-F]{64}$/;
    const mnemonicRegex = /^([a-z]+\s){11}[a-z]+$/;
    if (privateKeyRegex.test(str)) {
        return new ethers.Wallet(str);
    } else if (mnemonicRegex.test(str)) {
        return ethers.Wallet.fromMnemonic(str);
    } else {
        throw new Error("Invalid input: not a private key or mnemonic");
    }
}

const deploymentPath = path.join(__dirname, "../deployment");
const getLastDeployment = async () => {
    await fs.ensureDir(deploymentPath);
    const prevDetails = await fs
        .readJSON(`${deploymentPath}/deployments.json`)
        .catch(() => ({}));
    return prevDetails;
};

const updateDeployment = async (newDeployment: {}) => {
    await fs.writeJSON(`${deploymentPath}/deployments.json`, newDeployment);
};

export const updateChainDeploymentInfo = async (
    chain: string,
    chainInfo: {}
) => {
    const lastDeployment = await getLastDeployment();
    const chainId = getChainId(chain);
    if (!chainId) throw new Error("Invalid chain");
    await updateDeployment({
        ...lastDeployment,
        [chainId]: {
            ...lastDeployment[chainId],
            ...chainInfo,
        },
    });
};