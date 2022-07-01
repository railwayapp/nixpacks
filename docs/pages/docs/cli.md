---
title: CLI
---

# {% $markdoc.frontmatter.title %}

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
