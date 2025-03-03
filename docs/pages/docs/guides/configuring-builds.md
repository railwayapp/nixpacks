---
title: Configuring Builds
---

# {% $markdoc.frontmatter.title %}

This guide goes over a few common configuration scenarios so you can quickly get
up and running. For a complete reference of what is possible, please see the
[read the file configuration docs](/docs/configuration/file).

## Change what providers are run

You can have more than just the auto-detected provider contribute to the build by adding a `"providers"` key to the `nixpacks.toml` file. `"..."` matches the auto-detected providers.

```toml
providers = ["...", "python"]
```

You can also override the auto-detected providers by leaving `"..."` out of the array.

```toml
# Only the go provider will be run
providers = ["go"]
```

## Install additional packages

You can easily install additional Nix or Apt packages so that they are available during the build or at runtime. Packages are typically installed in the setup phase.

```toml
[phases.setup]
nixPkgs = ["...", "ffmpeg"] # Install the ffmpeg package from Nix
nixLibs = ["...", "gcc-unwrapped"] # Install the gcc-unwrapped package from Nix and add it to the LD_LIBRARY_PATH
aptPkgs = ["...", "wget"]   # Install the wget package with apt-get
```

The `"..."` item in the array is important as it extends the packages that will
be installed as opposed to overrideing them. This means that packages from the
provider (e.g. Node, Cargo, Python) will also installed.

It is recommended to install packages from Nix rather than Apt if they are available. You can search for Nix packages [here](https://search.nixos.org/packages?channel=unstable).

It's common for these to be many packages with a similar name and it can be hard to determine which one to pick. Here are some tips:

- If your application is shelling out to a command (i.e. your application expects a specific binary to be available on `$PATH`) look for a package that the binary you need in the `Programs provided` and add it as a `nixPkgs` entry.
- If your application needs a shared library, look for a package that contains a `dev` output. You can target just that `dev` (shared library) output by adding the suffix `.dev` to the `nixLibs` definition.

## Custom build command

You can override the build command with

```toml
[phases.build]
cmds = ["echo building!"]
```

Or you can add commands that will be run before or after the commands set by the providers.

```toml
[phases.build]
cmds = ["echo first", "...", "echo last"]
```

The same can be done to customize the commands for other phases.

## Custom start command

Override the command that is run when your container starts by setting the `start.cmd` value.

```toml
[start]
cmd = "./start.sh"
```

## New phase

Provider will typically define setup, install, and build phases. But you can add as many as you want. The following example will lint before the build and run tests afterwards.

```toml
[phases.lint]
cmds = ["yarn run lint"]
dependsOn = ["install"]

[phases.build]
dependsOn = ["...", "lint"]

[phases.test]
cmds = ["yarn run test"]
dependsOn = ["build"]
```

## Multiple Architectures

Nixpacks does not support multiple architectures in a single build.

If you need to build for multiple architectures, you will need to create a separate build for each architecture and then generate your own multi-architecture build using `docker manifest`. There's a [GitHub action which does this](https://github.com/iloveitaly/github-action-nixpacks) and is a great example to pattern off if you need a multi-architecture build.
