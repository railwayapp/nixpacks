#!/usr/bin/env node
import { compileTemplate } from "./config/template.mjs";
import { chmodRecursive } from "./util/chmodRecursive.mjs";
import { checkEnvErrors, isLaravel } from "./util/laravel.mjs";
import Logger from "./util/logger.mjs";
import { access, constants } from 'node:fs/promises'

const serverLogger = new Logger('server');

await access('/app/storage', constants.R_OK)
    .then(() => chmodRecursive('/app/storage'))
    .catch(() => { });

if (process.argv.length != 4) {
    new Logger('prestart').error(`Usage: ${process.argv[1]} <config-file> <output-file>`)
    process.exit(1);
}

if (isLaravel()) {
   checkEnvErrors('/app')
}

await compileTemplate(process.argv[2], process.argv[3])
serverLogger.info(`Server starting on port ${process.env.PORT}`)
