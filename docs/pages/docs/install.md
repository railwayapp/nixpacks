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
curl -fsSL https://raw.githubusercontent.com/railwayapp/nixpacks/master/install.sh | bash
```

## Scoop

Install Nixpacks from Scoop using the [official bucket](https://github.com/ScoopInstaller/Main/blob/master/bucket/nixpacks.json) (Windows Only)

```powershell
scoop install nixpacks
```

## Source

Build and install from source using [Rust](https://www.rust-lang.org/tools/install).

```sh
cargo install nixpacks
```
