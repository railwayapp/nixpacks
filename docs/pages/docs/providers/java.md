---
title: Java
---

# {% $markdoc.frontmatter.title %}

Java is detected if a `pom.[xml|atom|clj|groovy|rb|scala|yaml|yml]` or `gradlew` file is found.

## Setup

### JDK

The following major JDK versions are available

- `21`
- `20`
- `19`
- `17` (Default)
- `11`
- `8`

The version can be overridden by setting the `NIXPACKS_JDK_VERSION` environment variable.

### Gradle

The following major Gradle versions are available

- `8` (Default)
- `7`
- `6`
- `5`
- `4`

The version can be overridden by setting the `NIXPACKS_GRADLE_VERSION` environment variable.

## Build

If Maven is found:

```
/bin/maven -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install
```

If Gradle is found:

```
./gradlew clean build -x check -x test
```

## Start

If Maven is found:

```
java $JAVA_OPTS -jar target/*jar
```

If Maven and Wildfly Swarm is found:

```
java -Dswarm.http.port=$PORT $JAVA_OPTS -jar target/*jar
```

If Maven and Spring Boot is found:

```
java -Dserver.port=$PORT $JAVA_OPTS -jar target/*jar
```

If Gradle is found:

```
java $JAVA_OPTS -jar $(ls -1 build/libs/*jar | grep -v plain)
```

If Gradle and Spring Boot is found:

```
java $JAVA_OPTS -jar -Dserver.port=$PORT $(ls -1 build/libs/*jar | grep -v plain)
```
