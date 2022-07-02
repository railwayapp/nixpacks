---
title: Deno
---

# {% $markdoc.frontmatter.title %}

Deno is detected if there is a `deno.json` file found or if any `.(j|t)s` file is found that imports something from [deno.land](https://deno.land).

Apps built with [Deno Fresh](https://fresh.deno.dev/) should work out of the box.

## Install

_None_

## Build

The deno provider will compile all the project with `deno compile`.

## Start

If a `start` task is found in `deno.json` then

```
deno task start
```

Otherwise

```
deno run --allow-all index.j|ts
```
