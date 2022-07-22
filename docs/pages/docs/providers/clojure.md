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

If `lein-ring plugin detected`

```
lein ring uberjar
```

Default
```
lein uberjar
```


## Start

```
mv /app/target/uberjar/*standalone.jar /app/target/*standalone.jar; java $JAVA_OPTS -jar /app/target/*standalone.jar
```