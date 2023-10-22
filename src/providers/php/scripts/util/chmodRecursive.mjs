import * as fs from 'node:fs/promises'

export async function chmodRecursive(path) {
    await fs.chmod(path, 0o777);
    for (const file of await fs.readdir(path, { recursive: true })) {
        await fs.chmod(file, ((await fs.stat(file)).isDirectory() ? 0o777 : 0o666));
    }
}
