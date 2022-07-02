---
title: Crystal
---

# {% $markdoc.frontmatter.title %}

[Crystal](https://crystal-lang.org/) is detected if a `shard.yml` file is found.

The latest version of Crystal from the [Nix unstable channel](https://search.nixos.org/packages?channel=unstable&show=crystal&from=0&size=50&sort=relevance&type=packages&query=crystal) is used.

## Install

```
shards install
```

## Build

```
shards build --release
```

## Start

The first target found in `shards.yml` is run.
