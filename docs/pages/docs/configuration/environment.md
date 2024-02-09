---
title: Environment
---

# {% $markdoc.frontmatter.title %}

Nixpacks can be configured via environment variables. All of these variables are prefixed with `NIXPACKS_`.

| Variable                      | Description                                                                                  |
| :---------------------------- | :------------------------------------------------------------------------------------------- |
| `NIXPACKS_INSTALL_CMD`        | Override the install command to use                                                          |
| `NIXPACKS_BUILD_CMD`          | Override the build command to use                                                            |
| `NIXPACKS_START_CMD`          | Override command to run when starting the container                                          |
| `NIXPACKS_PKGS`               | Add additional [Nix packages](https://search.nixos.org/packages?channel=unstable) to install |
| `NIXPACKS_APT_PKGS`           | Add additional Apt packages to install (comma delimited)                                     |
| `NIXPACKS_LIBS`               | Add additional Nix libraries to make available                                               |
| `NIXPACKS_INSTALL_CACHE_DIRS` | Add additional directories to cache during the install phase                                 |
| `NIXPACKS_BUILD_CACHE_DIRS`   | Add additional directories to cache during the build phase                                   |
| `NIXPACKS_NO_CACHE`           | Disable caching for the build                                                                |
| `NIXPACKS_CONFIG_FILE`        | Location of the Nixpacks configuration file relative to the root of the app                  |
| `NIXPACKS_DEBIAN`             | Enable Debian base image, used for supporting OpenSSL 1.1                                    |
