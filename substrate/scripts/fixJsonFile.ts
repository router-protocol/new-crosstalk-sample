import fs from "fs";

async function fixJsonFile(filePath: string) {
  let content = fs.readFileSync(filePath, "utf-8");
  content = content.replace(/,\s*}/g, "}");
  fs.writeFileSync(filePath, content, "utf-8");
}
export default async function main(): Promise<void> {
  // to fix issue that @727-ventures/typechain-types creates
  fixJsonFile(`${__dirname}/../types/data/test_dapp.json`);
  fixJsonFile(`${__dirname}/../types/data/rate_limit.json`);
}

main()
  .then(() => {})
  .catch(console.log);
