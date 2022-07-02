---
title: Haskell
---

# {% $markdoc.frontmatter.title %}

Haskell with Stack is detected if your project has a `package.yaml` file and any `.hs` source files.

## Install

```sh
stack setup
```

## Build

```sh
stack build
```

## Start

Assumes that `package.yaml` has a list of `executables`.

```sh
stack run $(head packageYaml.executables)
```
