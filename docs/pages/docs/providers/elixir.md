---
title: Elixir
---

# {% $markdoc.frontmatter.title %}

Elixir is detected if a `mix.exs` file is found.

## Setup
```
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