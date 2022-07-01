---
title: Configuration
---

# {% $markdoc.frontmatter.title %}

Nixpacks can be configured via environment variables. Most of these variables are prefixed with `NIXPACKS_`.

| Variable               | Description                                                                                  |
| :--------------------- | :------------------------------------------------------------------------------------------- |
| `NIXPACKS_INSTALL_CMD` | Override the install command to use                                                          |
| `NIXPACKS_BUILD_CMD`   | Override the build command to use                                                            |
| `NIXPACKS_START_CMD`   | Override command to run when starting the container                                          |
| `NIXPACKS_PKGS_CMD`    | Add additional [Nix packages](https://search.nixos.org/packages?channel=unstable) to install |
| `NIXPACKS_APT_CMD`     | Add additional Apt packages to install                                                       |
| `NIXPACKS_LIBS_CMD`    | Add additional Nix libraries to make available                                               |
