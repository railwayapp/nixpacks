---
title: Zig
---

# {% $markdoc.frontmatter.title %}

Zig is detected if a `*.zig` or `gyro.zzz` file is found.

## Install

If a `gyro.zzz` file is detected then Gyro is downloaded.

## Build

```
zig build -Drelease-safe=true
```

## Start

```
./zig-out/bin/{name}
```
