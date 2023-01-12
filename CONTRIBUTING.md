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

When debugging it can be useful to see the intermediate files that Nixpacks generates (e.g. `Dockerfile`) You can do this my saving the build artifact to a specific directory instead of to a temp dir.

```
cargo run -- build examples/node --out test
```

_The `test` directory will contain everything that would be built with Docker. All the files that Nixpacks generates are in `.nixpacks`. You can manually build the image with `docker build test -f test/.nixpacks/Dockerfile`_.

## Snapshot Tests

Nixpacks uses [insta](https://github.com/mitsuhiko/insta) for snapshot tests. We use snapshot tests to generate and compare all build plans for the test apps in `examples/`. If a snapshot test fails due to a change to a provider, that is okay. It just means the snapshot needs to be reviewed and accepted. To test and review all snapshots, you can

First install insta

```
cargo install cargo-insta
```

Test and review the generate plan tests.

```
cargo insta test --review -- --test generate_plan_tests
# or
cargo snapshot
```

The snapshots are checked into CI and are reviewed as part of the PR. They ensure that a change to one part of Nixpacks does not unexpectedly change an unrelated part.

[Read the docs](https://insta.rs/docs/) for more information on cargo insta.

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
