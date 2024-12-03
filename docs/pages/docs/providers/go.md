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
- `1.23`

The version is parsed from the `go.mod` file.

## Install

If a `go.mod` file is found

```
go get
```

## Build

If your project has multiple binaries, you can specify which one to run with the `NIXPACKS_GO_BIN` environment variable.
Otherwise, the first binary found in the project's root directory or the project's `cmd` directory will be used.

```
go build -o out
# Or if there are no .go files in the root directory
go build -o out ./cmd/{name}

```

## Start

If the binary is built with cgo disabled then the binary is copied to a slim image to run in.

```
./out
```

## Caching

These directories are cached between builds

- Install and Build: `~/.cache/go-build`
