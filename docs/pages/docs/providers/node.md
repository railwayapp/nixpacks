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
- `NIXPACKS_MOON_APP_NAME`: Provide a name of the app you want to build from your moon repo.
- `NIXPACKS_NX_APP_NAME`: Provide a name of the NX app you want to build from your NX Monorepo
- `NIXPACKS_TURBO_APP_NAME`: Provide the name of the app you want to build from your Turborepo, if there is no `start` pipeline.

## Setup

The following major versions are available

- `16`
- `18` (Default)
- `20`
- `22`
- `23`

The version can be overridden by

- Setting the `NIXPACKS_NODE_VERSION` environment variable
- Specifying the `engines.node` field in `package.json`
- Creating a `.nvmrc` file in your project and specify the version or alias (`lts/*`)

Only a major version can be specified. For example, `18.x` or `20`.

**Node Canvas**

If [node-canvas](https://www.npmjs.com/package/canvas) is found in the `package.json` file, then the `libuuid` and `libGL` libraries are made available in the environment.

## Install

All dependencies found in `package.json` are installed with either NPM, Yarn, PNPM, or Bun (depending on `packageManager` field in package.json if present, or the detected lockfile).

## Build

The build script found in `package.json` if it exists.

- Or, if it's an NX Monorepo (detected if `nx.json` exists), the `build` pipeline for the `NIXPACKS_NX_APP_NAME` app will be called. Otherwise, it will run build for the `default_project` in `nx.json`. The build command is `(npm|pnpm|yarn|bun) run build <NxAppName>:build:production`.

- Or, if it's a Turborepo monorepo (detected if `turbo.json` exists), the `build` pipeline will be called (if it exists). Otherwise, the `build` script of the `package.json` referenced by `NIXPACKS_TURBO_APP_NAME` will be called, if `NIXPACKS_TURBO_APP_NAME` is provided. Otherwise, it will fall back to the build script found in `package.json` at the monorepos root.

- Or, if it's a [moon repo](https://moonrepo.dev/moon) (detected if `.moon/workspace.yml` exists), the `build` task for the `NIXPACKS_MOON_APP_NAME` will be called. The task name can be customized with `NIXPACKS_MOON_BUILD_TASK`. This will run the command `moon run <app_name>:<build_task>`.

## Start

The start command priority is:

- If it's a [moon repo](https://moonrepo.dev/moon)
  - It will use `NIXPACKS_MOON_APP_NAME` for the app name if provided, otherwise falls through to the next step.
  - It will use `NIXPACKS_MOON_BUILD_TASK` or `build` for the task to run.
- If it's an NX Monorepo
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
- Build (if its a moon repo): `.moon/cache`
- Build (if its an NX Monorepo): `<outputPathForApp>`

### Custom cache directories

You can specify `cacheDirectories` in `package.json`. Each directory that is provided in that field will be added to the build-time cache.

## Corepack

Nixpacks has first class support for [Corepack](https://nodejs.org/api/corepack.html), an experimental tool that enables installing specific versions of Node based package managers.

For example, To install the latest version of PNPM, add a `packageManager` key to your `package.json` file

```json
{
  "packageManager": "pnpm@latest"
}
```

Corepack will only be used on Node 16 and above.

## Bun Support

We support Bun as a stable package manager and runtime.

## SPA Application Support

If we detect your application is using [Vite](https://vite.dev) and doesn't have a server, we will automatically compile your app and run it using [Caddy](https://caddyserver.com/)

If you wish to turn off Caddy, you can set the environment variable `NIXPACKS_SPA_CADDY` to `false`.

If you have an application that doesn't pass the requirements for automatically using [Caddy](https://caddyserver.com/), set the `NIXPACKS_SPA_OUT_DIR` variable to the out directory of your application.

### Caddy requirements

If your package.json has `vite` any of the following dependencies:

- `react`
- `react-router` (but not in framework mode)
- `vue`
- `svelte` (but not `@sveltejs/kit`)
- `preact`
- `lit`
- `solid-js`
- `@builder.io/qwik`
