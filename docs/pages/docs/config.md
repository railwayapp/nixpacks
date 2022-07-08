---
title: Configuration
---

# {% $markdoc.frontmatter.title %}

## Environment Variables

Nixpacks can be configured via environment variables. Most of these variables are prefixed with `NIXPACKS_`.

| Variable                      | Description                                                                                  |
| :---------------------------- | :------------------------------------------------------------------------------------------- |
| `NIXPACKS_INSTALL_CMD`        | Override the install command to use                                                          |
| `NIXPACKS_BUILD_CMD`          | Override the build command to use                                                            |
| `NIXPACKS_START_CMD`          | Override command to run when starting the container                                          |
| `NIXPACKS_PKGS`               | Add additional [Nix packages](https://search.nixos.org/packages?channel=unstable) to install |
| `NIXPACKS_APT_PKGS`           | Add additional Apt packages to install                                                       |
| `NIXPACKS_LIBS`               | Add additional Nix libraries to make available                                               |
| `NIXPACKS_INSTALL_CACHE_DIRS` | Add additional directories to cache during the install phase                                 |
| `NIXPACKS_BUILD_CACHE_DIRS`   | Add additional directories to cache during the build phase                                   |

## Procfiles

The standard Procfile format is supported by Nixpacks. However, only a single process is supported. The command specified in the Procfile will override the provider start command.

```
web: npm run start
```

## Caching

By default Nixpacks providers will cache directories during the install and build phases. The specific directories are provider specific but are typically used to speed up installs (e.g. `~/.npm`) and builds (e.g. `~/.cache/go-build`). The contents of these directories are restored before the install/build phases are run and cleared afterwards. This means that the contents of the cached directories **do not appear in the final image**.

The default cache identifier is a hash of the absolute path to the directory being built. This means that subsequent builds of the same directory will be faster out of the box. You can override the cache identifier by passing a `--cache-key` value to the `build` command.

Caching can be disabled entirely by passing `--no-cache`.
