import { task } from "hardhat/config";
import { TaskArguments } from "hardhat/types";
import { ethers, providers } from "ethers";
import { ChainType } from "./../utils/types";
import { getSignerFromPrivateKeyOrMnemonic } from "../utils/utils";
import { getChainId, getChainInfo, getUChainList } from "../utils/chain";
import { Args, enrollForEachChains } from "../utils/OnEachChain";
import { enrollOnSingleChain as enrollAOnSingleChain } from "./enroll_on_chain";

async function enrollOnSingleChain(
    signer: ethers.Signer,
    chain: ChainType,
    args: Args, // chainlist entered by user
    enrollWith: string[]
) {
    const uChainList = Array.from(new Set([...args.chainList, ...enrollWith]));

    await enrollAOnSingleChain(
        signer,
        chain,
        {
            chainList: uChainList,
            contractList: args.contractList,
        },
        []
    );
}

async function enrollOtherWithAddOnChain(
    signer: ethers.Signer,
    chain: ChainType,
    args: Args, // chainlist entered by user
    enrollWith: string[]
) {
    await enrollAOnSingleChain(
        signer,
        chain,
        {
            chainList: enrollWith,
            contractList: args.contractList,
        },
        []
    );
}

task("ENROLLADDED_ONEACH", "enroll contracts on provided chains")
    //   .addParam('contractlist', 'Description of contract list parameter')
    .addParam("chainlist", "list of newly added chain")
    .addParam(
        "enrollwith",
        "list of chain to which newly added chain should be linked"
    )
    .addOptionalParam(
        "pkm",
        "Description of private key or mnemonic as parameter"
    )
    .addOptionalParam(
        "contractlist",
        "contract list for which enrolling to chainlist should be done"
    )
    .setAction(async (taskArgs: TaskArguments, hre: any) => {
        const { chainlist, pkm, enrollwith, contractlist } = taskArgs;
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

        const enrollWith: string[] = Array.from(
            new Set(enrollwith.trim().split(" "))
        );

        chainList.map((chain: string) => {
            if (!getChainId(chain))
                throw new Error(`invalid chain provided ${chain}`);
        });
        enrollWith.map((chain: string) => {
            if (!getChainId(chain))
                throw new Error(`invalid chain provided ${chain}`);
        });

        const contractList: string[] = contractlist
            ? Array.from(new Set(contractlist.trim().split(" ")))
            : [];
        if (!contractList.length) contractList.push("XERC1155"); // by default
        await enrollForEachChains(
            enrollOnSingleChain,
            {
                chainList: getUChainList(chainList),
                contractList,
            },
            signer,
            getUChainList(enrollWith)
        );

        console.log("Enrolling enrollwith chains to add on");
        await enrollForEachChains(
            enrollOtherWithAddOnChain,
            {
                chainList: getUChainList(enrollWith),
                contractList,
            },
            signer,
            getUChainList(chainList)
        );
    });