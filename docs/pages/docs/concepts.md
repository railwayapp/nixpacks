---
title: Concepts
---

# {% $markdoc.frontmatter.title %}

This is how Nixpacks works at a high level.

## How Nix is used

Nix packages are used for OS and language level dependencies (e.g. [nodejs](https://search.nixos.org/packages?channel=unstable&show=nodejs&from=0&size=50&sort=relevance&type=packages&query=nodejs) and [ffmpeg](https://search.nixos.org/packages?channel=unstable&show=ffmpeg&from=0&size=50&sort=relevance&type=packages&query=ffmpeg)). These packages are built and loaded into the environment where we then use these dependencies to install, build, and run the app (e.g. `npm install`, `cargo build`, etc.).

## How Docker is used

At the moment nixpacks generates a `Dockerfile` based on all information available. To create an image this is then built with `docker build`. However, this may change so providers should not need to know about the underlying Docker implementation.
