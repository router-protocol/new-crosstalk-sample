// chainType -> chainId -> Data
export const chainIdsMap: { [chainId: string]: any } = {
    '80001': {
        chainId: '80001',
        key: 'polygonMumbai',
        rpc: 'https://polygon-mumbai.g.alchemy.com/v2/8NUj-xxS1p3mJMFBo0h_szwvmUhMjkH4',
        chainName: 'Polygon Mumbai',
        gateway: '0x3ddf956f27297cd1E4423E97D7DDF2552f539C2F',
    },
    '5': {
        chainId: '5',
        key: 'goerli',
        rpc: 'https://goerli.infura.io/v3/91531d5460e34331a77e37156c61e223',
        chainName: 'Goerli',
        gateway: '0x',
    },
    '43113': {
        chainId: '43113',
        key: 'avalancheFuji',
        rpc: 'https://avalanche-fuji.infura.io/v3/91531d5460e34331a77e37156c61e223',
        chainName: 'Avalanche Fuji',
        gateway: '0x1aD8B95ee94Ae3Ef08Ebf777403572eB99D0E1Aa',
    },
    '9000': {
        chainId: '9000',
        key: 'routerTestnet',
        rpc: 'https://devnet-alpha.evm.rpc.routerprotocol.com/',
        chainName: 'Router Testnet',
    },
};

export const chainKeysMap: { [chainKey: string]: string } = {
    goerli: '5',
    avalancheFuji: '43113',
    polygonMumbai: '80001',
    routerTestnet: '9000',
};

export function getChainInfo(chain: string) {
    if (chainKeysMap[chain]) return chainIdsMap[chainKeysMap[chain]];
    return chainIdsMap[chain] || null;
}

export function getChainId(chain: string): string | null {
    return getChainInfo(chain)?.chainId;
}

export function getChainKey(chain: string): string | null {
    return getChainInfo(chain)?.key;
}

export function getUChainList(chains: string[]): string[] {
    const array: string[] = [];
    chains.map((chain) => {
        const chainId = getChainId(chain);
        if (chainId) array.push(chainId);
    });
    return Array.from(new Set(array));
}