---
title: Node
---

# {% $markdoc.frontmatter.title %}

The Node provider supports NPM, Yarn, Yarn 2, and PNPM.

## Variables

The Node provider sets the following environment variables:

- `NODE_ENV=production`
- `NPM_CONFIG_PRODUCTION=false`: Ensure that dev deps are always installed

## Install

All dependencies found in `packages.json` are installed with either NPM, Yarn, or PNPM.

## Build

The build script found in `package.json` if it exists.

## Start

The start command priority is

- Start script in `package.json`
- Main file
- `index.js`
