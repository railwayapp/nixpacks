import { readFile, writeFile } from "fs/promises";
import { getNixPath } from "../util/nix.mjs";

const replaceStr = input =>
    input
        // If statements
        .replaceAll(/\$if\s*\((\w+)\)\s*\(([^]*?)\)\s*else\s*\(([^]*?)\)/gm,
            (_all, condition, value, otherwise) =>
                process.env[condition] ? replaceStr(value) : replaceStr(otherwise)
        )
        // Variables
        .replaceAll(/\${(\w+)}/g,
            (_all, name) => process.env[name]
        )
        // Nix paths
        .replaceAll(/\$!{(\w+)}/g,
            (_all, exe) => getNixPath(exe)
        )

export async function compileTemplate(infile, outfile) {
    await writeFile(outfile,
        replaceStr(await readFile(infile, { encoding: 'utf8' })),
        { encoding: 'utf8' })
}
