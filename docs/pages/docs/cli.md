---
title: CLI
---

# {% $markdoc.frontmatter.title %}

The main Nixpacks commands are `build` and `plan`.

## Build

Create an image from an app source directory. The resulting image can then be run using Docker.

For example

```sh
nixpacks build ./path/to/app --name my-app
```

View all build options with

```sh
nixpacks build --help
```

### Options

|                             |                                                                |
| :-------------------------- | :------------------------------------------------------------- |
| `--install-cmd <cmd>`, `-i` | Specify the install command                                    |
| `--build-cmd <cmd>`, `-b`   | Specify the buildcommand                                       |
| `--start-cmd <cmd>`, `-s`   | Specify the install command                                    |
| `--name <name>`             | Name for the built image                                       |
| `--env <envs...>`           | Provide environment variables to your build.                   |
| `--pkgs <pkgs...>`, `-p`    | Provide additional Nix packages to install in the environment  |
| `--apt <pkgs...>`           | Provide additional apt packages to install in the environment  |
| `--libs <libs...>`          | Provide additional Nix libraries to install in the environment |
| `--tag <tag...>`, `-t`      | Additional tags to add to the output image                     |
| `--label <labels...>`, `-l` | Additional labels to add to the output image                   |
| `--cache-key <key>`         | Unique identifier to use for the build cache                   |
| `--no-cache`                | Disable caching for the build                                  |
| `--out <dir>`, `-o`         | Save output directory instead of building it with Docker       |

#### Environment Variables

Environment variables can be provided in the format `FOO` or `FOO=bar`. If no equal sign is present then the value is pulled from the current environment.

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
