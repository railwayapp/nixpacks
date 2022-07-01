---
title: Getting Started
---

# {% $markdoc.frontmatter.title %}

To get started with Nixpacks you need an app that you want to build and package. You can bring your own or use one of the [many examples on GitHub](https://github.com/railwayapp/nixpacks/tree/main/examples).

## 1. Install

```
brew install railwayapp/tap/nixpacks
```

[View more installation options](/docs/install)

## 2. Build and package

```
nixpacks build ./path/to/app --name my-app
```

This creates an image with the name `my-app`.

## 3. Run the built image

```
docker run -it my-app
```

![Getting Started](/images/getting-started.png)
