import { task } from "hardhat/config";
import { TaskArguments } from "hardhat/types";
import ethers from "ethers";
import { ChainType } from "./../utils/types";
import {
    getChainDeployment,
    getSignerFromPrivateKeyOrMnemonic,
} from "../utils/utils";
import { getChainId } from "../utils/chain";
import { Args, enrollForEachChains } from "../utils/OnEachChain";

export async function enrollOnSingleChain(
    signer: ethers.Signer,
    chain: ChainType,
    args: Args, // chainlist entered by user
    addOn: string[]
) {
    const hre = require("hardhat");
    const provider = signer.provider;
    if (!provider) throw new Error("signer provider is undefined");

    const network = await provider.getNetwork();
    hre.provider = provider;
    hre.network.provider = provider;
    const { ethers } = hre;

    for (const contract_key of args.contractList) {
        console.log(`Enrolling remote contract for ${contract_key}...`);
        const contract_factory = await ethers.getContractFactory(contract_key);
        //@ts-ignore
        const contract_address = (await getChainDeployment(network.name))[
            contract_key
        ];
        if (!contract_address) throw new Error("contract address undefined");

        const contract = await (
            await contract_factory.attach(contract_address)
        ).connect(signer);
        for (const chain of args.chainList) {
            if (getChainId(chain) == getChainId(network.name)) continue; // not enrolling same contract

            //@ts-ignore
            const remote_contract_address = (await getChainDeployment(chain))[
                contract_key
            ];
            if (!remote_contract_address)
                throw new Error("remote contract address undefined");

            console.log(`EnrollRemoteContract[${network.name}-> ${chain}]: started `);
            const tx = await contract
                .connect(signer)
                .setContractOnChain(0, getChainId(chain), remote_contract_address, {
                    gasLimit: 50000,
                });
            console.log(
                `EnrollRemoteContract[${network.name} -> ${chain}]: tx send with hash `,
                tx.hash
            );
            await tx.wait();
            console.log(
                `EnrollRemoteContract[${network.name} -> ${chain}]: tx went successfully`
            );
        }
        console.log(`Enrolling of remote contract for ${contract_key} completed`);
    }
}
task("ENROLL_ONEACH", "enroll contracts on provided chains")
    .addParam("chainlist", "Description of chainlist parameter")
    .addOptionalParam(
        "contractlist",
        "contract list for which enrolling to chainlist should be done"
    )
    .addOptionalParam(
        "pkm",
        "Description of private key or mnemonic as parameter"
    )
    .setAction(async (taskArgs: TaskArguments, hre: any) => {
        const { chainlist, pkm, contractlist } = taskArgs;
        let signer;
        if (pkm) signer = getSignerFromPrivateKeyOrMnemonic(pkm);
        else {
            // load from env
            const morp = process.env.MNEMONIC || process.env.PRIVATE_KEY;
            if (!morp) throw new Error("Provide mnemonic or private key");
            signer = getSignerFromPrivateKeyOrMnemonic(morp);
        }

        // signer = (await hre.ethers.getSigners())[0];

        const chainList: string[] = Array.from(
            new Set(chainlist.trim().split(" "))
        );

        const contractList: string[] = contractlist
            ? Array.from(new Set(contractlist.trim().split(" ")))
            : [];
        if (!contractList.length) contractList.push("XERC1155"); // by default

        chainList.map((chain: string) => {
            if (!getChainId(chain)) throw new Error("invalid chain provided");
        });

        await enrollForEachChains(
            enrollOnSingleChain,
            {
                chainList,
                contractList,
            },
            signer,
            []
        );
    });