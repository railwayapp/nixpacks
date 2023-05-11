---
title: Clojure
---

# {% $markdoc.frontmatter.title %}

Clojure is detected if a `project.clj` or `build.clj` file is found.

## Setup

The following JDK versions are available

- `8` (Default)
- `11`
- `latest`

The version can be overridden by

- Setting the `NIXPACKS_JDK_VERSION` environment variable
- Setting the version in a `.jdk-version` file

## Build

If a `build.clj` file for [`tools.build`](https://clojure.org/guides/tools_build) is found:

```
clojure -T:build uber; if [ -f /app/target/uberjar/*standalone.jar ]; then mv /app/target/uberjar/*standalone.jar /app/target/*standalone.jar; fi
```

If the `lein-ring` plugin is found:

```
lein ring uberjar; if [ -f /app/target/uberjar/*standalone.jar ]; then mv /app/target/uberjar/standalone.jar /app/target/*standalone.jar; fi
```

Default

```
lein uberjar; if [ -f /app/target/uberjar/*standalone.jar ]; then mv /app/target/uberjar/standalone.jar /app/target/*standalone.jar; fi
```

## Start

```
java $JAVA_OPTS -jar /app/target/*standalone.jar
```
