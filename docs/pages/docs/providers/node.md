---
title: Node
---

# {% $markdoc.frontmatter.title %}

The Node provider supports NPM, Yarn, Yarn 2, PNPM and Bun.

## Environment Variables

The Node provider sets the following environment variables:

- `CI=true`
- `NODE_ENV=production`
- `NPM_CONFIG_PRODUCTION=false`: Ensure that dev deps are always installed
- `NIXPACKS_NX_APP_NAME`: Provide a name of the NX app you want to build from your NX Monorepo
- `NIXPACKS_TURBO_APP_NAME`: Provide the name of the app you want to build from your Turborepo, if there is no `start` pipeline.

## Setup

The following major versions are available

- `14`
- `16` (Default)
- `18`

The version can be overridden by

- Setting the `NIXPACKS_NODE_VERSION` environment variable
- Specifying the `engines.node` field in `package.json`

Only a major version can be specified. For example, `14.x` or `14`.

**Node Canvas**

If [node-canvas](https://www.npmjs.com/package/canvas) is found in the `package.json` file, then the `libuuid` and `libGL` libraries are made available in the environment.

## Install

All dependencies found in `package.json` are installed with either NPM, Yarn, PNPM, or Bun (depending on the lockfile detected).

## Build

The build script found in `package.json` if it exists.

Or, if it's an NX Monorepo (detected if `nx.json` existis), the `build` pipeline for the `NIXPACKS_NX_APP_NAME` app will be called. Otherwise, it will run build for the `default_project` in `nx.json`. The build command is `(npm|pnpm|yarn|bun) run build <NxAppName>:build:production`.

Or, if it's a Turborepo monorepo (detected if `turbo.json` exists), the `build` pipeline will be called (if it exists). Otherwise, the `build` script of the `package.json` referenced by `NIXPACKS_TURBO_APP_NAME` will be called, if `NIXPACKS_TURBO_APP_NAME` is provided. Otherwise, it will fall back to the build script found in `package.json` at the monorepos root.

## Start

The start command priority is

- If its an NX Monorepo
  - It will use `NIXPACKS_NX_APP_NAME` for the app name if provided, otherwise it will use the `default_project` from `nx.json`
  - If the app has a `start` target `npx nx run <appName>:start:production` or just `npx nx run <appName>:start` if no production configuration is present
  - If the app is a NextJS project: `npm run start`
  - If `targets.build.options.main` exists in the apps `Project.json`: `node <outputPath>/<mainFileName>.js` (e.g `node dist/apps/my-app/main.js`)
  - Fallback: `node <outputPath>/index.js` (e.g `node dist/apps/my-app/index.js`)
- If Turborepo is detected
  - If a `start` pipeline exists, call that;
  - Otherwise, if `NIXPACKS_TURBO_APP_NAME` is provided, call the `start` script of that package;
  - Otherwise, run `npx turbo run start`, which will simply run all `start` scripts in the monorepo in parallel.
- Start script in `package.json`
- Main file
- `index.js`

## Caching

These directories are cached between builds

- Install: Global NPM/Yarn/PNPM cache directories
- Install (if Cypress detected): `~/.cache/Cypress`
- Build: `node_modules/.cache`
- Build (if NextJS detected): `.next/cache`
- Build (if its an NX Monorepo): `<outputPathForApp>`

### Custom cache directories

You can specify `cacheDirectories` in `package.json`. Each directory that is provided in that field will be added to the build-time cache.

## Corepack

Nixpacks has first class support for [Corepack](https://nodejs.org/api/corepack.html), an experimental tool that enables installing specific versions of Node based package managers.

For example, To install a specific version of PNPM, add a `packageManager` key to your `package.json` file

```json
{
  "packageManager": "pnpm@7.7.0"
}
```

Corepack will only be used on Node 16 and above.

## Bun Support

We support Bun, but due to Bun being in alpha, it is unstable and very experimental.
