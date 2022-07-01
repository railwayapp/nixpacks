---
title: How It Works
---

# {% $markdoc.frontmatter.title %}

Nixpacks works in two steps

## 1. Plan

Analyze the app source directory and generates a reproducible build plan. This plan can be saved (in JSON format) and re-used at a later date to build the image in the exact same way every time.

To create the plan, language providers are matched against the app source directory and suggest Nix packages, an install command, build command, and start command. All of these can be overwritten by the user.

## 2. Build

The build step takes the build plan and creates an OCI compliant image (with Docker) that can be deployed and run anywhere. This happens in the following steps

1. Create build plan
2. Copy app source to temp directory
3. Use the Nix packages in the build plan and generate an `environment.nix` file
4. Build the app in multiple phases
   - **Setup**: Install all necessary Nix packages
   - **Install**: Download all build dependencies
   - **Build**: Generate everything necessary to run the app
   - **Start**: Configure a default command to run when starting the container
5. Done!

Overall the process is fairly simple.

## How Nix is used

Nix packages are used for OS and language level dependencies (e.g. [nodejs](https://search.nixos.org/packages?channel=unstable&show=nodejs&from=0&size=50&sort=relevance&type=packages&query=nodejs) and [ffmpeg](https://search.nixos.org/packages?channel=unstable&show=ffmpeg&from=0&size=50&sort=relevance&type=packages&query=ffmpeg)). These packages are built and loaded into the environment where we then use these dependencies to install, build, and run the app (e.g. `npm install`, `cargo build`, etc.).

## How Docker is used

At the moment nixpacks generates a `Dockerfile` based on all information available. To create an image this is then built with `docker build`. However, this may change so providers should not need to know about the underlying Docker implementation.
