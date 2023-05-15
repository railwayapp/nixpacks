---
title: Lunatic
---

# {% $markdoc.frontmatter.title %}

[Lunatic](https://github.com/lunatic-solutions/) is detected if both

- a `Cargo.toml` file is found and
- [`.cargo/config.toml`](https://doc.rust-lang.org/cargo/reference/config.html) has the `runner = "lunatic"`

For example `.cargo/config.toml`:

```toml
[build]
target = "wasm32-wasi"

[target.wasm32-wasi]
runner = "lunatic"
```

## Environment Variables

None

## Setup

By default the latest stable version of Rust available in this [rust-overlay](https://github.com/oxalica/rust-overlay) is used. The version can be overridden with either

- a `.rust-version` file
- The `rust-version` property of `Cargo.toml`
- setting the `NIXPACKS_RUST_VERSION` environment variable
- A `rust-toolchain.toml` file

## Install

_None_

## Build

```
cargo build --release
```

## Start

The binaries get .wasm suffix and will be ran with the lunatic runtime.

If your project has multiple binaries, you can specify which one to run with the `NIXPACKS_RUST_BIN` environment variable.
Optionally, it can be overriden with the `default_run` property in `Cargo.toml` under the `[package]` section.

```
./target/wasm32-wasi/release/{name}.wasm
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
