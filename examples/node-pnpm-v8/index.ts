import { execSync } from "child_process";

const pnpmVersion = execSync("pnpm --version", { encoding: "utf-8" }).trim();
const pnpmMajorVersion = pnpmVersion.split(".")[0];
console.log(`Hello from PNPM ${pnpmMajorVersion}`);
