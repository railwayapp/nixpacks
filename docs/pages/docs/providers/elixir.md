---
title: Elixir
---

# {% $markdoc.frontmatter.title %}

Elixir is detected if a `mix.exs` file is found.

## Setup
The following Elixir versions are available

- `latest`  (Default)
- `1.12`
- `1.11`
- `1.10`
- `1.9`

The version can be overridden by

- Setting the `NIXPACKS_ELIXIR_VERSION` environment variable
- Setting the version in a `.elixir-version` file



```
MIX_ENV=prod 

mix local.hex --force
mix local.rebar --force
mix deps.get --only prod
```

## Build

```
mix compile
mix assets.deploy
```


## Start

```
mix phx.server
```