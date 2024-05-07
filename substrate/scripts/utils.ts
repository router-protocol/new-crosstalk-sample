import { ApiPromise } from "@polkadot/api";
import fse from "fs-extra";
import fs from "fs";

export function parseCommandLineArgs(): any {
  const parsedArgs: any = {};

  const args: string[] = process.argv;
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg.startsWith("--")) parsedArgs[arg.slice(2)] = args[i + 1];
  }
  return parsedArgs;
}

export class DeploymentStore {
  storePath: string = `deployment/deployments.json`;

  constructor(storePath?: string) {
    if (storePath) this.storePath = storePath;
    (async () => {
      try {
        const fileExists = await fse.pathExists(this.storePath);
        if (!fileExists) {
          await fse.ensureDir("deployment");
          await fse.writeJSON(this.storePath, {}, { spaces: 2 });
        }
      } catch (error) {
        console.error("Error creating file:", error);
      }
    })();
  }

  async getStore() {
    return await fse.readJSON(this.storePath);
  }

  async pGetInfo(pkey: string) {
    const info = await this.getStore();
    return info[pkey] ? info[pkey] : {};
  }

  async psGetInfo(pkey: string, skey: string) {
    const pinfo = await this.pGetInfo(pkey);
    return pinfo[skey] ? pinfo[skey] : {};
  }

  async pskGetInfo(pkey: string, skey: string, tkey: string) {
    const psinfo = await this.psGetInfo(pkey, skey);
    return psinfo[tkey] ? psinfo[tkey] : {};
  }

  async store(
    pkey: string, //primary key
    skey: string, //secondary key
    kname: string, //key name
    proxyAddr: string,
    implementationAddr: string,
    data?: Object
  ) {
    let deploymentData = await JSON.parse(
      fs.readFileSync(this.storePath, "utf-8")
    );
    if (pkey && !deploymentData[pkey]) deploymentData[pkey] = {};
    if (skey && !deploymentData[pkey][skey]) deploymentData[pkey][skey] = {};

    if (pkey && skey && kname)
      deploymentData[pkey][skey][kname] = data
        ? data
        : {
            proxy: proxyAddr,
            implementation: [implementationAddr],
            creationTime: Date.now(),
            updatedTime: [Date.now()],
          };
    else if (pkey && skey)
      deploymentData[pkey][skey] = data
        ? data
        : {
            proxy: proxyAddr,
            implementation: [implementationAddr],
            creationTime: Date.now(),
            updatedTime: [Date.now()],
          };
    else if (pkey)
      deploymentData[pkey] = data
        ? data
        : {
            proxy: proxyAddr,
            implementation: [implementationAddr],
            creationTime: Date.now(),
            updatedTime: [Date.now()],
          };
    else
      deploymentData = data
        ? data
        : {
            proxy: proxyAddr,
            implementation: [implementationAddr],
            creationTime: Date.now(),
            updatedTime: [Date.now()],
          };

    fs.writeFileSync(this.storePath, JSON.stringify(deploymentData));
  }
}

export function ss558AccountIdToHexEncodedString(
  api: ApiPromise,
  accountId: string
): string {
  return (
    "0x" +
    Buffer.from(
      api.registry.createType("AccountId", accountId).toU8a()
    ).toString("hex")
  );
}

export function ss558AccountIdFromHexEncodedString(
  api: ApiPromise,
  accountId: string
): string {
  return api.registry.createType("AccountId", accountId).toString();
}
