import { execSync } from "child_process";

export const e = cmd => execSync(cmd).toString().replace('\n', '');