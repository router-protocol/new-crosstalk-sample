export interface ReturnType {
    [key: string]: string;
}

export interface ChainType {
    rpc: string;
    chainName: string;
    chainId: string;
    key: string;
    gateway: string;
}

export interface ContractInfo {
    feePayer?: string;
    PingPing?: string;
    XERC1155?: string;
    contractlist?: string;
}

export interface JsonType {
    [chainId: string]: {
        [key: string]: ContractInfo;
    };
}