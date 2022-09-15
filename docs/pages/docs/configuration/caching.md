---
title: Caching
---

# {% $markdoc.frontmatter.title %}

By default Nixpacks providers will cache directories during the install and build phases. The specific directories are provider specific but are typically used to speed up installs (e.g. `~/.npm`) and builds (e.g. `~/.cache/go-build`). The contents of these directories are restored before the install/build phases are run and cleared afterwards. This means that the contents of the cached directories **do not appear in the final image**.

The default cache identifier is a hash of the absolute path to the directory being built. This means that subsequent builds of the same directory will be faster out of the box. You can override the cache identifier by passing a `--cache-key` value to the `build` command.

Caching can be disabled entirely by passing `--no-cache`.

Passing`--inline-cache` will write cache metadata into the output image.

Using previous image -created with inline cache enabled- as a cache source, Can be achieved by passing `--cache-from`.
