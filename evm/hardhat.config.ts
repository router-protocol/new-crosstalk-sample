import { HardhatUserConfig } from "hardhat/config";
import "@nomiclabs/hardhat-waffle";
import "@typechain/hardhat";
import "@openzeppelin/hardhat-upgrades";
import "solidity-coverage";
import { resolve } from "path";
import { config as dotenvConfig } from "dotenv";
import { NetworkUserConfig } from "hardhat/types";
import "@nomiclabs/hardhat-etherscan";
import "./tasks";

dotenvConfig({ path: resolve(__dirname, "./.env") });

const chainIds = {
  ganache: 5777,
  goerli: 5,
  hardhat: 7545,
  kovan: 42,
  mainnet: 1,
  rinkeby: 4,
  bscTestnet: 97,
  bsc: 56,
  ropsten: 3,
  mumbai: 80001,
  avalanche: 43114,
  polygon: 137,
  fuji: 43113,
  arbitrum: 42161,
  arbitrum_rinkeby: 421611,
  fantom_testnet: 4002,
  optimism: 10,
  optimism_kovan: 69,
  fantom: 250,
  harmony: 1666600000,
  cronos: 25,
  aurora: 1313161554,
  kava: 2222,
  stardust: 588,
  moonbeam: 1284,
};

// Ensure that we have all the environment variables we need.
const mnemonic = process.env.MNEMONIC;
if (!mnemonic) {
  throw new Error("Please set your MNEMONIC in a .env file");
}

const infuraApiKey = process.env.INFURA_API_KEY;
if (!infuraApiKey) {
  throw new Error("Please set your INFURA_API_KEY in a .env file");
}

function getChainConfig(network: keyof typeof chainIds): NetworkUserConfig {
  let url = "";
  url = "https://" + network + ".infura.io/v3/" + infuraApiKey;
  switch (network) {
    case "polygon":
      url =
        "https://polygon-mainnet.g.alchemy.com/v2/hCz4x1BLpLDP3NoomXivfaqND37qCSgS";
      break;
    case "mumbai":
      url = "https://matic-mumbai.chainstacklabs.com";
      break;
    case "bsc":
      url = "https://bsc-dataseed.binance.org/";
      break;
    case "avalanche":
      url = "https://api.avax.network/ext/bc/C/rpc";
      break;
    case "arbitrum":
      url =
        "https://arbitrum-mainnet.infura.io/v3/fd9c5dbc69de41048405e7072cda9bf9";
      break;
    case "optimism":
      url = "https://mainnet.optimism.io";
      break;
    case "fantom":
      url = "https://rpc.ankr.com/fantom";
      break;
    case "mainnet":
      url = "https://mainnet.infura.io/v3/0d73cc5bbe184146957a9d00764db99f";
      break;
    case "harmony":
      url = "https://api.harmony.one";
      break;
    case "aurora":
      url = "https://mainnet.aurora.dev";
      break;
    case "cronos":
      url = "https://rpc.artemisone.org/cronos";
      break;
    case "kava":
      url = "https://evm.kava.io";
      break;
    case "stardust":
      url = "https://stardust.metis.io/?owner=588";
      break;
    case "moonbeam":
      url = "https://moonbeam.api.onfinality.io/public";
      break;
    case "fuji":
      url = "https://rpc.ankr.com/avalanche_fuji";
      break;
    case "goerli":
      url = "https://goerli.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161";
      break;
    case "bscTestnet":
      url = "https://bsc-testnet.public.blastapi.io";
      break;
  }

  return {
    // accounts: {
    //   count: 10,
    //   mnemonic,
    //   path: "m/44'/60'/0'/0",
    //   initialIndex:2,
    // },
    accounts: [`${process.env.PRIVATE_KEY}`],
    chainId: chainIds[network],
    url,
    // gas: 10000000,
    // // gasPrice: network == "bsc" ? 20000000000 : 200000000000,
    // gasPrice: 4500000000,
  };
}

const config = {
  defaultNetwork: "hardhat",
  gasReporter: {
    currency: "USD",
    enabled: process.env.REPORT_GAS ? true : false,
    excludeContracts: [],
    src: "./contracts",
  },
  networks: {
    hardhat: {},
    ropsten: {
      saveDeployments: true,
      accounts: {
        initialIndex: 0,
        mnemonic,
        // path: "m/44'/60'/0'/0",
      },
      chainId: chainIds["ropsten"],
      url: "https://ropsten.infura.io/v3/" + infuraApiKey + "",
    },
    kovan: getChainConfig("kovan"),
    polygon: getChainConfig("polygon"),
    bsc: getChainConfig("bsc"),
    avalanche: getChainConfig("avalanche"),
    arbitrum: getChainConfig("arbitrum"),
    fantom: getChainConfig("fantom"),
    optimism: getChainConfig("optimism"),
    mainnet: getChainConfig("mainnet"),
    harmony: getChainConfig("harmony"),
    aurora: getChainConfig("aurora"),
    cronos: getChainConfig("cronos"),
    kava: getChainConfig("kava"),
    stardust: getChainConfig("stardust"),
    moonbeam: getChainConfig("moonbeam"),
    fuji: getChainConfig("fuji"),
    goerli: getChainConfig("goerli"),
    mumbai: getChainConfig("mumbai"),
    bsctestnet: getChainConfig("bscTestnet"),
  },
  paths: {
    artifacts: "./artifacts",
    cache: "./cache",
    sources: "./contracts",
    tests: "./test",
    // deploy: "./deploy",
    deployments: "./deployments",
    // imports: "./imports",
  },
  solidity: {
    version: "0.8.9",
    settings: {
      evmVersion: "berlin",
      metadata: {
        // Not including the metadata hash
        // https://github.com/paulrberg/solidity-template/issues/31
        bytecodeHash: "none",
      },
      // You should disable the optimizer when debugging
      // https://hardhat.org/hardhat-network/#solidity-optimizer-support
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },

  typechain: {
    outDir: "typechain",
    target: "ethers-v5",
  },
  namedAccounts: {
    deployer: 0,
  },
  etherscan: {
    apiKey: {
      kovan: process.env.MAINNET_ETHERSCAN_KEY,
      polygonMumbai: process.env.POLYGONSCAN_API_KEY,
      polygon: process.env.POLYGONSCAN_API_KEY,
      bsc: process.env.BSC_ETHERSCAN_KEY,
      avalanche: "QAE2JD7XIBCYB6Z6GSKNJIHKZ8XGVYM8AI",
      opera: process.env.FTMSCAN_KEY,
      arbitrumOne: process.env.ARBITRUM_KEY,
      optimisticEthereum: process.env.OPTIMISM_KEY,
      mainnet: process.env.ETH_ETHERSCAN_KEY,
      aurora: process.env.AURORA_KEY,
      harmony: process.env.HARMONY_KEY,
      moonbeam: process.env.MOONBEAM_ETHERSCAN_KEY,
      kava: process.env.MOONBEAM_ETHERSCAN_KEY,
      goerli: process.env.ETHERSCAN_API_KEY,
      avalancheFujiTestnet: "QAE2JD7XIBCYB6Z6GSKNJIHKZ8XGVYM8AI",
      bscTestnet: process.env.BSCSCAN_API_KEY,
    },

    customChains: [
      {
        network: "cronos",
        chainId: 25,
        urls: {
          apiURL: "https://api.cronoscan.com/api",
          browserURL: "https://cronoscan.com/",
        },
      },
      {
        network: "kava",
        chainId: 2222,
        urls: {
          apiURL: "https://explorer.kava.io/api",
          browserURL: "https://explorer.kava.io/",
        },
      },
      {
        network: "moonbeam",
        chainId: 1284,
        urls: {
          apiURL: "https://api-moonbeam.moonscan.io/api/",
          browserURL: "https://moonscan.io/",
        },
      },
    ],
  },
};

export default config;
