---
title: Installation
---

# {% $markdoc.frontmatter.title %}

## Homebrew

Install Nixpacks with [Homebrew](https://brew.sh/) (macOS Only)

```sh
brew install nixpacks
```

## MacPorts

Install Nixpacks with [MacPorts](https://www.macports.org/) (macOS Only)

```sh
sudo port install nixpacks
```

## Curl

Download Nixpacks from GH releases and install automatically

```sh
curl -sSL https://nixpacks.com/install.sh | bash
```

This works across all supported platforms.

## PowerShell

Download Nixpacks from GH release and install automatically (with powershell)

```ps1
irm https://nixpacks.com/install.ps1 | iex
```

## Debian (and derivatives like Ubuntu)

Nixpacks provides a `.deb` file provided in each [release](https://github.com/railwayapp/nixpacks/releases).

```sh
# Replace {VERSION} with the version that you want to install
curl -LO https://github.com/railwayapp/nixpacks/releases/download/v{VERSION}/nixpacks-v{VERSION}-amd64.deb
sudo dpkg -i nixpacks-v{VERSION}-amd64.deb
```

## Scoop

Install Nixpacks from Scoop using the [official bucket](https://github.com/ScoopInstaller/Main/blob/master/bucket/nixpacks.json) (Windows Only)

```powershell
scoop install nixpacks
```

## Windows

Nixpacks provides `.msi` files provided in each [release](https://github.com/railwayapp/nixpacks/releases).

## Source

Build and install from source using [Rust](https://www.rust-lang.org/tools/install).

> Nixpacks currently requires a [Rust](https://www.rust-lang.org/tools/install) version no lower than [1.60](https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html)

```sh
cargo install nixpacks
```
