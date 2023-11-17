import { e } from "./cmd.mjs";

export const getNixPath = (exe) => e(`nix-store -q ${e(`which ${exe}`)}`);
