---
title: Gleam
---

# {% $markdoc.frontmatter.title %}

Gleam is detected if both `gleam.toml` and `manifest.toml` are found.

## Setup & Install

Nixpacks will detect the Gleam version your project uses from your `manifest.toml`. It will also install Erlang, Elixir and Rebar3.

## Build

Nixpacks will export a BEAM build of your project.

## Run

Nixpacks will run the exported BEAM build using the provided `entrypoint.sh`.
