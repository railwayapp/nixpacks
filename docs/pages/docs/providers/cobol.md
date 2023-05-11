---
title: COBOL
---

# {% $markdoc.frontmatter.title %}

## Environment Variables

To configure the COBOL provider you can use the following environment variables:

- `NIXPACKS_COBOL_COMPILE_ARGS`: Provide custom `cobc` arguments
- `NIXPACKS_COBOL_APP_NAME`: Provide the name the cobol file to compile

## Setup

The COBOL provider uses [GnuCOBOL](https://gnucobol.sourceforge.io/)

## Install

GnuCOBOL and gcc are installed

## Build

the following command is used ( see section below to see how the arguments are generated ):
`cobc <cobcArgs> ./<appName> <path>`

### `cobcArgs`

- If `NIXPACKS_COBOL_COMPILE_ARGS` is set that is used`
- Otherwise `-x -o` is used

### `appName`

- First if `NIXPACKS_COBOL_APP_NAME` is set that is used.
- next the source files are searched for the presence of an `index.cbl`. If one is found `index` is used
- Lastly the source files are searched for any file with the `cbl` extension. If one is found the file name is used.

### `path`

- The source files are searched for `*<appName>.cbl` if found that path is used.

## Start

`./app-name` is run

## Caching

GnuCOBOL and gcc are cached between builds
