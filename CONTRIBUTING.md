# Contributing

Thanks for your interest in contributing to this project! PRs and issues are welcome, but please keep in mind that this project is still and alpha and the the implementation details and API will most likely change between now and a stable release.

For larger changes, please first make an [RFC issue](https://github.com/railwayapp/nixpacks/issues). You can follow along with the roadmap in the [GitHub project](https://github.com/railwayapp/nixpacks/projects/1).

## How to contribute

First, make sure you can build and run the project

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) and [Docker](https://docs.docker.com/get-docker/) installed.
1. Checkout this repo `git clone https://github.com/railwayapp/nixpacks.git`
1. Build the source `cargo build`
1. Run the tests `cargo test`
1. Build an example `cargo run -- build examples/node --name node`
1. Run the example `docker run node`

You should see `Hello from Node` printed to the console.

## Debugging

When debugging it can be useful to see the `environment.nix` and `Dockerfile` generated. You can do this my saving the build artifact to a specific directory instead of to a temp dir.

```
cargo run -- build examples/node --out test
```

_The `test` directory will contain everything that would be built with Docker._

## Contribution Ideas

The easiest way to contribute is to add support for new languages. There is a list of languages we would like to add [here](https://github.com/railwayapp/nixpacks/issues?q=is%3Aissue+is%3Aopen+label%3A%22new+provider%22), but languages not on the list are welcome as well. To guage interest you can always create an issue before working on an implementation.

## Making PRs

To make a PR follow [GitHubs guide](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request).

PRs are all checked with

- `cargo check`
- `cargo test`
- `cargo clippy`

so you can run these locally to ensure CI passes.

Most PRs should include tests.
