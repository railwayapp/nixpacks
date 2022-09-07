---
title: Java
---

# {% $markdoc.frontmatter.title %}

Java is detected if a `pom.[xml|atom|clj|groovy|rb|scala|yaml|yml]` or `gradlew` file is found.

## Install

```
Skipped
```

## Build

If maven is found: 
```
/bin/maven -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install
```

If gradle is found:
```
./gradlew build
```

## Start

```
{start_command_from_pom.*}
```
