---
title: C#
---

# {% $markdoc.frontmatter.title %}

CSharp is detected if any `*.csproj*` files are found.

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
