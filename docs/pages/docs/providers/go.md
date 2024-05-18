---
title: Go
---

# {% $markdoc.frontmatter.title %}

Go is detected if a `main.go` file is found.

## Environment Variables

- `CGO_ENABLED=0`: Build a statically linkable binary

## Setup

The following Go versions are available:

- `1.18`
- `1.19`
- `1.20`
- `1.21`
- `1.22` (default)

The version is parsed from the `go.mod` file.

## Install

If a `go.mod` file is found

```
go get
```

## Build

```
go build -o out
```

## Start

If the binary is built with cgo disabled then the binary is copied to a slim image to run in.

```
./out
```

## Caching

These directories are cached between builds

- Install and Build: `~/.cache/go-build`
