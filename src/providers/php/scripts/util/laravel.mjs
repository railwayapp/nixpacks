import Logger from "./logger.mjs"
import * as fs from 'node:fs/promises'
import * as path from 'node:path'

const variableHints = {
    'APP_ENV': 'You should probably set this to `production`.'
};

const logger = new Logger('laravel');

export const isLaravel = () => process.env['IS_LARAVEL'] != null;

function checkVariable(name) {
    if (!process.env[name]) {
        let hint =
            `Your app configuration references the ${name} environment variable, but it is not set.`
            + (variableHints[name] ?? '');

        logger.warn(hint);
    }
}

export async function checkEnvErrors(srcdir) {
    const envRegex = /env\(["']([^,]*)["']\)/g;
    const configDir = path.join(srcdir, 'config');

    const config =
        (await Promise.all(
            (await fs.readdir(configDir))
                .filter(fileName => fileName.endsWith('.php'))
                .map(fileName => fs.readFile(path.join(configDir, fileName)))
        )).join('');

    for (const match of config.matchAll(envRegex)) {
        if (match[1] != 'APP_KEY') checkVariable(match[1]);
    }

    if (!process.env.APP_KEY) {
        logger.warn('Your app key is not set! Please set a random 32-character string in your APP_KEY environment variable. This can be easily generated with `openssl rand -hex 16`.');
    }
}
