---
title: Getting Started
---

# {% $markdoc.frontmatter.title %}

To get started with Nixpacks you need an app that you want to build and package. You can bring your own or use one of the [many examples on GitHub](https://github.com/railwayapp/nixpacks/tree/main/examples).

## 1. Install

```
brew install nixpacks
```

[View more installation options](/docs/install)

## 2. Build and package

```
nixpacks build ./path/to/app --name my-app
```

This creates an image with the name `my-app`.

Nixpacks allows you to customize all options that are used to build the image. For example, you can add additional system packages and specify the build and start commands.

```
nixpacks build ./path/to/app --name my-app \
                             --pkgs cowsay \
                             --build-cmd ./build.sh \
                             --start-cmd "echo hello | cowsay"
```

## 3. Run the image

```
docker run -it my-app
```

![Getting Started](/images/getting-started.png)
