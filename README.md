# Nixpacks

[![CI](https://github.com/railwayapp/bb/actions/workflows/ci.yml/badge.svg)](https://github.com/railwayapp/bb/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/nixpacks)](https://crates.io/crates/nixpacks)
[![Rust: 1.57+](https://img.shields.io/badge/rust-1.57+-93450a)](https://blog.rust-lang.org/2021/12/02/Rust-1.57.0.html)

**App source + Nix packages + Docker = Image**

Nixpacks takes a source directory and produces an OCI compliant image that can be deployed anywhere. The project was started by the [Railway](https://railway.app) team as an alternative to [Buildpacks](https://buildpacks.io/) and attempts to address a lot of the shortcomings and issues that occurred when deploying thousands of user apps to the Railway platform. The biggest change is that system and language dependencies are pulled from the Nix ecosystem.

Read the docs 👉 [nixpacks.com](https://nixpacks.com)

## Contributing

Contributions are welcome with the big caveat that this is a very early stage project and the implementation details and API will most likely change between now and a stable release. For more details on how to contribute, please see the [Contributing guidelines](./CONTRIBUTING.md).
