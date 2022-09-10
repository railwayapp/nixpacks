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

If Maven is found: 
```
/bin/maven -DoutputFile=target/mvn-dependency-list.log -B -DskipTests clean dependency:list install
```

If Gradle is found:
```
./gradlew build
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
java $JAVA_OPTS -jar build/libs/*.jar
```


If Gradle and Spring Boot is found:
```
java $JAVA_OPTS -jar -Dserver.port=$PORT" build/libs/*.jar
```
