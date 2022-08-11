---
title: Installation
---

# {% $markdoc.frontmatter.title %}

## Homebrew

Install Nixpacks with [Homebrew](https://brew.sh/) (MacOS Only)

```sh
brew install railwayapp/tap/nixpacks
```

## Curl

Download Nixpacks from GH releases and install automatically

```sh
curl -sSL https://nixpacks.com/install.sh | bash
```

## Scoop

Install Nixpacks from Scoop using the [official bucket](https://github.com/ScoopInstaller/Main/blob/master/bucket/nixpacks.json) (Windows Only)

```powershell
scoop install nixpacks
```

## Source

Build and install from source using [Rust](https://www.rust-lang.org/tools/install).

> Nixpacks currently requires a [Rust](https://www.rust-lang.org/tools/install) version no lower than [1.57](https://blog.rust-lang.org/2021/12/02/Rust-1.57.0.html)

```sh
cargo install nixpacks
```
