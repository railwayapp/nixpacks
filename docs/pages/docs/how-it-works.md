---
title: How It Works
---

# {% $markdoc.frontmatter.title %}

Nixpacks works in two steps

## Plan

Analyze the app source directory and generate a reproducible build plan. This plan can be saved (in JSON format) and re-used at a later date to build the image in the exact same way every time.

To create the plan, language providers are matched against the app source directory and suggest Nix packages, an install command, build command, and start command. All of these can be overwritten by the user.

## Build

The build step takes the build plan and creates an [OCI-compliant](https://opencontainers.org/about/overview/) image (with Docker BuildKit) that can be deployed and run anywhere. This happens in the following steps

1. Create a build plan
2. Copy app source to temp directory
3. Run through each phase in topological order. Each phase will do one or many of the following
   - Install Nix and/or Apt packages
   - Run shell commands
   - Add assets to the image
4. Configure a default command to run when starting the container
5. Done!

Overall, the process is fairly simple.

### Phases

There can be any number of phases that run as part of the build. Phases can also depend on other phases and the order they run in ensures that phases are run after any phases that they depend on.

Most providers create a build plan with the following common phases

- **Setup**: Install all necessary Nix packages
- **Install**: Download all build dependencies
- **Build**: Generate everything necessary to run the app

However, the capabilities of each phase is identical.

## How Nix is used

Nix packages are used for OS and language level dependencies (e.g., [nodejs](https://search.nixos.org/packages?channel=unstable&show=nodejs&from=0&size=50&sort=relevance&type=packages&query=nodejs) and [ffmpeg](https://search.nixos.org/packages?channel=unstable&show=ffmpeg&from=0&size=50&sort=relevance&type=packages&query=ffmpeg)). These packages are built and loaded into the environment where we then use these dependencies to install, build, and run the app (e.g. `npm install`, `cargo build`, etc.).

## How Docker is used

At the moment, nixpacks generates a `Dockerfile` based on all information available. To create an image this is then built with `docker build`. However, this may change, so providers should not need to know about the underlying Docker implementation.
