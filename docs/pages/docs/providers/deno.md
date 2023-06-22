---
title: Deno
---

# {% $markdoc.frontmatter.title %}

Deno is detected if there is a `deno.{json,jsonc}` file found or if any `.{ts,tsx,js,jsx}` file is found that imports something from [deno.land](https://deno.land).

Apps built with [Deno Fresh](https://fresh.deno.dev/) should work out of the box.

## Install

_None_

## Build

The deno provider will compile all the projects with `deno compile`.

## Start

If a `start` task is found in `deno.{json,jsonc}` then:

```
deno task start
```

Otherwise, the first file matching `index.{ts,tsx,js,jsx}` pattern, eg.:

```
deno run --allow-all index.ts
```
