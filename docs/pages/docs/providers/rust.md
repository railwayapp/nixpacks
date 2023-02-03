---
title: Rust
---

# {% $markdoc.frontmatter.title %}

Rust is detected if a `Cargo.toml` file is found.

## Environment Variables

- `ROCKET_ADDRESS=0.0.0.0`: Allows [Rocket](https://rocket.rs) apps to accept non-local connections

## Setup

By default the latest stable version of Rust available in this [rust-overlay](https://github.com/oxalica/rust-overlay) is used. The version can be overridden with either

- a `.rust-version` file
- The `rust-version` property of `Cargo.toml`
- setting the `NIXPACKS_RUST_VERSION` environment variable
- A `rust-toolchain.toml` file

## Install

_None_

## Build

The Rust provider will build for the musl target by default so that a statically
linked binary can be created. This creates a much smaller image size. However,
if you do not want to build for the musl target you can set `NIXPACKS_NO_MUSL=1`.

```
cargo build --release
```

## Start

If your project has multiple binaries, you can specify which one to run with the `NIXPACKS_RUST_BIN` environment variable.
Optionally, it can override with the `default_run` property in `Cargo.toml` under the `[package]` section.

```
./target/release/{name}
```

## Caching

These directories are cached between builds

- Build: `~/.cargo/git`
- Build: `~/.cargo/registry`
- Build: `target`

## Workspaces

Nixpacks will auto-detect if you are using [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).
This checks `workspace.default_members` first and then `workspace.members`.
It also respects the `workspace.exclude` field.

To set which workspace Nixpacks will build, just set the `NIXPACKS_CARGO_WORKSPACE`
environment variable and Nixpacks will use it as the `--package` argument.
