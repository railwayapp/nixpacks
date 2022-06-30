# Nixpacks

[![CI](https://github.com/railwayapp/bb/actions/workflows/ci.yml/badge.svg)](https://github.com/railwayapp/bb/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/nixpacks)](https://crates.io/crates/nixpacks)

**App source + Nix packages + Docker = Image**

Nixpacks takes a source directory and produces an OCI compliant image that can be deployed anywhere. The project was started by the [Railway](https://railway.app) team as an alternative to [Buildpacks](https://buildpacks.io/) and attempts to address a lot of the shortcomings and issues that occurred when deploying thousands of user apps to the Railway platform. The biggest change is that system and language dependencies are pulled from the Nix ecosystem, which provides a bunch of benefits.

You can follow along with the roadmap in the [GitHub project](https://github.com/railwayapp/nixpacks/projects/1).

## Core Ideas

- ✨ **Intuitive defaults**: In most cases, building and deploying an app with nixpacks should _just work_ with no configuration needed.
- ⚙️ **Customization where necessary**: Every part of the pipeline should be customizable. These include the [Nix packages](https://search.nixos.org/packages) to add to the environment and build/start commands.
- 🚀 **Easily extendible**: New providers (languages) should be able to be easily added to nixpacks with minimal knowledge of Nix and Docker.

## How Nix is used

Nix packages are used for OS and language level dependencies (e.g. [nodejs](https://search.nixos.org/packages?channel=unstable&show=nodejs&from=0&size=50&sort=relevance&type=packages&query=nodejs) and [ffmpeg](https://search.nixos.org/packages?channel=unstable&show=ffmpeg&from=0&size=50&sort=relevance&type=packages&query=ffmpeg)). These packages are built and loaded into the environment where we then use these dependencies to install, build, and run the app (e.g. `npm install`, `cargo build`, etc.).

## How Docker is used

At the moment nixpacks generates a `Dockerfile` based on all information available. To create an image this is then built with `docker build`. However, this may change so providers should not need to know about the underlying Docker implementation.

# Getting Started

1. [Install Nixpacks](#installation)
2. Build an image from app source code `nixpacks build ~/path/to/source --name my-app`
3. Run the image `docker run -it my-app`

_Note: Docker must be running and available locally to use Nixpacks_

# Language Support

At the moment Nixpacks supports the following languages out of the box

- [Node, NPM, and Yarn](./docs/node.md)
- [Go](./docs/go.md)
- [Rust](./docs/rust.md)
- [Deno/Fresh](./docs/deno.md)
- [Haskell with Stack](./docs/haskell-stack.md)
- [Zig](./docs/zig.md)
- [Ruby/Rails](./docs/ruby.md)
- [Python/Django](./docs/python.md)
- [PHP/Laravel](./docs/php.md)
- [Dart](./docs/dart.md)
- [C#/DotNet](./docs/csharp.md)
- [F#](./docs/fsharp.md)
- [Java/Spring](./docs/java.md)
- [StaticHTML](./docs/static.md)
- [Crystal](./docs/crystal.md)

# Installation

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

# Environment Variables

Environment variables can be made available to the install, build, and start phases of Nixpacks with the `--env` flag. For example

```sh
nixpacks build . --env "HELLO=world" "FOO"
```

If no equal sign is present, then the value is pulled from the current environment.

# CLI Reference

The main Nixpacks commands are `build` and `plan`.

## Build

Create an image from an app source directory. The resulting image can then be run using Docker.

For example

```sh
nixpacks build ./path/to/app --name my-app --env "HELLO=world" --pkgs cowsay
```

View all build options with

```sh
nixpacks build --help
```

## Plan

The plan command will show the full set of options (nix packages, build cmd, start cmd, etc) that will be used to when building the app. This plan can be saved and used to build the app with the same configuration at a future date.

For example,

```sh
nixpacks plan examples/node
```

View all plan options with

```sh
nixpacks plan --help
```

## Help

For a full list of CLI commands run

```sh
nixpacks --help
```

## How this works

Nixpacks works in two steps

### Plan

Analyze the app source directory and generates a reproducible build plan. This plan can be saved (in JSON format) and re-used at a later date to build the image in the exact same way every time.

Language providers are matched against the app source directory and suggest Nix packages, an install command, build command, and start command. All of these can be overwritten by the user.

### Build

The build step takes the build plan and creates an OCI compliant image (with Docker) that can be deployed and run anywhere. This happens in the following steps

1. Create build plan
2. Copy app source to temp directory
3. Use the Nix packages in the build plan and generate an `environment.nix` file
4. Build the app in multiple phases
   - **Setup**: Install all necessary Nix packages
   - **Install**: Download all build dependencies
   - **Build**: Generate everything necessary to run the app
   - **Start**: Configure a default command to run when starting the container
5. Done!

Overall the process is fairly simple.

## Contributing

Contributions are welcome with the big caveat that this is a very early stage project and the implementation details and API will most likely change between now and a stable release. For more details on how to contribute, please see the [Contributing guidelines](./CONTRIBUTING.md).
