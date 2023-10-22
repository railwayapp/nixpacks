import { execSync } from "node:child_process";

const e = cmd => execSync(cmd).toString().replace('\n', '');

export const getNixPath = (exe) => e(`nix-store -q ${e(`which ${exe}`)}`);
