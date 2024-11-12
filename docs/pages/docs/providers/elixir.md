---
title: Elixir
---

# {% $markdoc.frontmatter.title %}

Elixir is detected if a `mix.exs` file is found.

## Setup

The following Elixir versions are available

- `latest` (Default)
- `1.17`
- `1.16`
- `1.15`
- `1.14`
- `1.13`
- `1.12`
- `1.11`
- `1.10`
- `1.9`

The version can be overridden by

- Elixir version is extracted from the `mix.exs` file automatically
- Setting the `NIXPACKS_ELIXIR_VERSION` environment variable
- Setting the version in a `.elixir-version` file

The OTP version is automatically set and cannot currently be customized.

The default install script is:

```shell
MIX_ENV=prod

mix local.hex --force
mix local.rebar --force
mix deps.get --only prod
```

## Build

```shell
mix compile
mix assets.deploy
mix ecto.deploy # if available
```

If you are building outside of a live environment, you may want to omit `ecto.deploy` (which can sometimes rely on a
database connection) which you can do by overriding the build command.

## Start

```shell
mix phx.server
```

## Environment Variables

The following environment variables are set by default:

```shell
MIX_ENV=prod
ELIXIR_ERL_OPTIONS="+fnu"
```
