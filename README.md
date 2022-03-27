# Better Buildpacks

App source + Nix packages + Docker = Image

## Usage

Create directory that can be built with Docker based on app source.

```
cargo run -- build examples/yarn
```

Show help

```
> cargo run -- build --help
Create a Docker build-able directory from app source

USAGE:
    bb build [OPTIONS] <PATH>

ARGS:
    <PATH>    App source

OPTIONS:
    -b, --build-cmd <build_cmd>    Specify the build command to use
        --dockerfile               Show the Dockerfile that would be generated
    -h, --help                     Print help information
        --nix                      Show the nix expression that would generated
    -s, --start-cmd <start_cmd>    Specify the start command to use
```

## Steps

1. Detect: Return the first matching builder for a source directory
2. Build
   1. Generate nix expression based on packages provided by builder
   2. Generate Dockerfile based on install, build, and start commands
   3. Copy app source to a temp directory
   4. Create `environment.nix` and `Dockerfile` files in the temp directory
   5. Return a Docker build command to run
