# Nixpacks

[![CI](https://github.com/railwayapp/bb/actions/workflows/ci.yml/badge.svg)](https://github.com/railwayapp/bb/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/nixpacks)](https://crates.io/crates/nixpacks)

**App source + Nix packages + Docker = Image**

Nixpacks takes a source directory and produces an OCI compliant image that can be deployed anywhere. The project was started by the [Railway](https://railway.app) team as an alternative to [Buildpacks](https://buildpacks.io/) and attempts to address a lot of the shortcomings and issues that occurred when deploying thousands of user apps to the Railway platform. The biggest change is that system and language dependencies are pulled from the Nix ecosystem, which provides a bunch of benefits.

You can follow along with the roadmap in the [GitHub project](https://github.com/railwayapp/nixpacks/projects/1).

## Core Ideas

- ✨ **Intutive defaults**: In most cases, building and deploying and app with nixpacks should _just work_ with no configuration needed.
- ⚙️ **Customization where necessary**: Every part of the pipeline should be customizable. These include the [Nix packages](https://search.nixos.org/packages) to add to the environment and build/start commands.
- 🚀 **Easily extendible**: New providers (languages) should be able to be easily added to nixpacks with minimal knowledge of Nix and Docker.

## How Nix is used

Nix packages are used for OS and language level dependencies (e.g. [nodejs](https://search.nixos.org/packages?channel=unstable&show=nodejs&from=0&size=50&sort=relevance&type=packages&query=nodejs) and [ffmpeg](https://search.nixos.org/packages?channel=unstable&show=ffmpeg&from=0&size=50&sort=relevance&type=packages&query=ffmpeg)). These packages are built and loaded into the environment where we then use these dependencies to install, build, and run the app (e.g. `npm install`, `cargo build`, etc.).

## How Docker is used

At the moment nixpacks generates a `Dockerfile` based on all information available. To create an image this is then built with `docker build`. However, this may change so providers should not need to know about the underlying Docker implementation.

# Docs

This project is not yet distributed anywhere and must be built with [Rust](https://www.rust-lang.org/tools/install).

1. Checkout this repo `git clone https://github.com/railwayapp/nixpacks.git`
2. Build the source `cargo build`
3. Run the tests `cargo test`

There are two main commands

### `plan`

Generates a build plan and outputs to stdout.

```
nixpacks plan $APP_SRC
```

![image](https://user-images.githubusercontent.com/3044853/165360487-550e51c4-198f-4a40-af23-6f498736b280.png)


View the help with `cargo run -- plan --help`

### `build`

Creates a runnable image with Docker

```
nixpacks build $APP_SRC --name $NAME
```

![image](https://user-images.githubusercontent.com/3044853/165363312-5c1d39c3-c461-4b87-b7a2-1f3f00957f01.png)

View the help with `cargo run -- build --help`

## How this works

Nixpacks works in two phases

**Plan**

Analyze the app source directory and generates a reproducible build plan. This plan can be saved (in JSON format) an re-used at a later date to build the image in the exact same way every time.

Language providers are matched against the app source directory and suggest Nix packages, an install command, build command, and start command. All of these can be overwritten by the user.

**Build**

The build phase takes the build plan and creates an OCI compliant image (with Docker) that can be deployed and run anywhere. This happens in the following steps

1. Create build plan
2. Copy app source to temp directory
3. Use the Nix packages in the build plan and generate an `environment.nix` file
4. Use the install, build, and start commands and generate a `Dockerfile`
5. Done!

Overall the process is fairy simple.

## Language providers

At the moment nixpacks supports the following languages out of the box

- [Node via NPM](https://www.npmjs.com/)
- [Node via Yarn](https://yarnpkg.com/)
- [Go](https://golang.org)
- [Rust](https://www.rust-lang.org/)
- [Deno](https://deno.land)

## Contributing

Contributions are welcome with the big caveat that this is a very early stage project and the implementation details and API will most likely change between now and a stable release. For more details on how to contribute, please see the [Contributing guidelines](./CONTRIBUTING.md).
