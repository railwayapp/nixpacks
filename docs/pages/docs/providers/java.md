---
title: Java
---

# {% $markdoc.frontmatter.title %}

Java is detected if a `pom.[xml|atom|clj|groovy|rb|scala|yaml|yml]` file is found.

## Install

```
Skipped
```

## Build

```
/bin/maven -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install
```

## Start

```
{start_command_from_pom.*}
```
