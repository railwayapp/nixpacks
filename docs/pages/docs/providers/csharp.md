---
title: C#
---

# {% $markdoc.frontmatter.title %}

CSharp is detected if any `*.csproj*` files are found.

The SDK version can be overridden by

- Setting the `NIXPACKS_CSHARP_SDK_VERSION` environment variable
- Setting the version in a `global.json` file

## Install

```
dotnet restore
```

## Build

```
dotnet publish --no-restore -c Release -o {}
```

## Start

```
./out
```
