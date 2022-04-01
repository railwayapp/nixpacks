# Nixpacks

[![CI](https://github.com/railwayapp/bb/actions/workflows/ci.yml/badge.svg)](https://github.com/railwayapp/bb/actions/workflows/ci.yml)

**App source + Nix packages + Docker = Image**

The goal of this project is to build an app source directory in a reproducible way. Providers analyze the source code and recommend nix packages and suggest install/build/start commands. However, all of these settings can be overriden by the user.

Nixpacks currently supports

- Node/NPM
- Yarn
- Go

More langauges will be added very soon

## Getting Started

_Note: This is a young project and is in active development_

This project is not yet distributed anywhere and must be built with [Rust](https://www.rust-lang.org/tools/install).

1. Checkout this repo `git clone https://github.com/railwayapp/nixpacks.git`
2. Build the source `cargo build`
3. Run the tests `cargo test`

There are two main commands

### `plan`

Generates a build plan and outputs to stdout.

```
cargo run -- plan $APP_SRC
```

![image](https://user-images.githubusercontent.com/3044853/161355091-1eb38fd7-aa59-412e-904d-74e48e2016e7.png)

View the help with `cargo run -- plan --help`

### `build`

Creates a runnable image with Docker

```
cargo run -- build $APP_SRC --name $NAME
```

![image](https://user-images.githubusercontent.com/3044853/161355162-73651b6d-6ee2-41ee-a0f0-abbf581ce8f4.png)


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

## Future Steps

This project is still in early development and is just the start.

- Lanauge support
  * [ ] NPM
  * [ ] Yarn
  * [ ] Golang
  * [ ] Python
  * [ ] Rust
  * [ ] Java
  * [ ] Zip
  * [ ] Crystal
  * [ ] Ruby
