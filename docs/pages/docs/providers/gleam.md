---
title: Gleam
---

# {% $markdoc.frontmatter.title %}

Gleam is detected if both `gleam.toml` and `manifest.toml` are found.

## Setup & Install

Nixpacks will detect the Gleam version your project uses from your `gleam.toml` if you fill in the optional `gleam` field with just a version number like `1.12.0`. It will also install Erlang, Elixir and Rebar3.

## Build

Nixpacks will export a BEAM build of your project.

## Run

Nixpacks will run the exported BEAM build using the provided `entrypoint.sh`.
