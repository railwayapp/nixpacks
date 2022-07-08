---
title: F#
---

# {% $markdoc.frontmatter.title %}

Fsharp is detected if a `*.fsproj` file is found.

## Install

```
dotnet restore
```

## Build

```
dotnet publish --no-restore -c Release -o {out_dir}
```

## Start

```
./out
```
