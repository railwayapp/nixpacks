---
title: Node
---

# {% $markdoc.frontmatter.title %}

The Node provider supports NPM, Yarn, Yarn 2, and PNPM.

## Environment Variables

The Node provider sets the following environment variables:

- `NODE_ENV=production`
- `NPM_CONFIG_PRODUCTION=false`: Ensure that dev deps are always installed

## Setup

The following major versions are available

- `10`
- `12`
- `14`
- `16` (Default)
- `18`

The version can be overriden by

- Setting the `NIXPACKS_NODE_VERSION` environment variable
- Specifying the `engines.node` field in `package.json`

Only a major version can be specified. For example, `14.x` or `14`.

## Install

All dependencies found in `packages.json` are installed with either NPM, Yarn, or PNPM.

## Build

The build script found in `package.json` if it exists.

**Node Canvas**

If [node-canvas](https://www.npmjs.com/package/canvas) is found in the `package.json` file, then the `libuuid` and `libGL` libraries are made available in the environment.

## Start

The start command priority is

- Start script in `package.json`
- Main file
- `index.js`

## Caching

These directories are cached between builds

- Install: Global NPM/Yarn/PNPM cache directories
- Install (if Cypress detected): `~/.cache/Cypress`
- Build: `node_modules/.cache`
- Build (if NextJS detected): `.next/cache`
