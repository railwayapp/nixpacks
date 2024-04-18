---
title: Swift
---

# {% $markdoc.frontmatter.title %}

## Setup

The following Swift versions are available

- `5.8` (Default)
- `5.7.3`
- `5.6.2`
- `5.5.3`
- `5.5.2`
- `5.4.2`
- `5.4`
- `5.1.1`
- `5.0.2`
- `5.0.1`
- `4.2.3`
- `4.2.1`
- `4.1`
- `4.1.3`
- `4.0.3`
- `3.1`
- `3.1.1`

The version can be overridden by

- Setting the version in a `.swift-version` file
- Specifying a `swift-tools-version` field in `Package.swift`

## Install

```
swift package resolve
```

## Build

```
swift build -c release --static-swift-stdlib
```

## Start

```
./.build/release/{name}
```
