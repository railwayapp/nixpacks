---
title: Scheme
---

# {% $markdoc.frontmatter.title %}

Scheme via [Haunt](https://dthompson.us/projects/haunt.html) is detected is there is a `haunt.scm` file found.

## Setup

Installs the basic dependencies

## Build

Build the project based on your `haunt.scm`.

## Start

Haunt wasn't made for this, so our entrypoint is actually a Guile script.  
Make sure it is in the root directory with your `haunt.scm`.  
Note: This is only necessary in production. You can use the `haunt` command normally in dev.

```scheme
;; init.scm
(system "haunt serve")
```
