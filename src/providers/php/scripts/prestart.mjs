#!/usr/bin/env node
import { compileTemplate } from "./config/template.mjs";
import { e } from "./util/cmd.mjs";
import { checkEnvErrors, isLaravel } from "./util/laravel.mjs";
import Logger from "./util/logger.mjs";
import { access, constants } from 'node:fs/promises'

const prestartLogger = new Logger('prestart');
const serverLogger = new Logger('server');

if (process.argv.length != 4) {
    prestartLogger.error(`Usage: ${process.argv[1]} <config-file> <output-file>`)
    process.exit(1);
}

await Promise.all([
    isLaravel() ? checkEnvErrors('/app') : Promise.resolve(),
    access('/app/storage', constants.R_OK)
        .then(() => e('chmod -R ugo+rw /app/storage'))
        .catch(() => {}),
    compileTemplate(process.argv[2], process.argv[3])
]).catch(err => prestartLogger.error(err));

serverLogger.info(`Server starting on port ${process.env.PORT}`)
