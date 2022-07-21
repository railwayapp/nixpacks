---
title: Clojure
---

# {% $markdoc.frontmatter.title %}

Clojure is detected if a `project.clj` file is found.

## Setup

The following JDK versions are available

- `8`  (Default)
- `11`
- `latest`

The version can be overriden by

- Setting the `NIXPACKS_JDK_VERSION` environment variable
- Setting the version in a `.jdk-version` file

## Build

```
lein uberjar
```


## Start

```
java $JAVA_OPTS -jar target/uberjar/*standalone.jar
```