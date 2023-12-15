---
title: Scala
---

# {% $markdoc.frontmatter.title %}

Currently sbt projects are supported for Scala. For gradle and maven support, please look
at Java. Scala is detected by `build.sbt` in project root.

## SBT Setup

### JDK

The following major JDK versions are available

- `21`
- `20`
- `19`
- `17` (Default)
- `11`
- `8`

The version can be overridden by setting the `NIXPACKS_JDK_VERSION` environment variable.

### Requirements

The project should contain the `sbt-native-packager` sbt plugin. This can be done
by adding the plugin to `project/plugins.sbt`.

```scala
// Check https://github.com/sbt/sbt-native-packager for version
addSbtPlugin("com.github.sbt" % "sbt-native-packager" % "x.x.x")
```

After that enable the `JavaAppPackaging` plugin in `build.sbt` and set the
`executableScriptName` to `main`. An example of the `build.sbt` can be seen here.

```scala
val scala3Version = "3.2.2"

lazy val root = project
  .in(file("."))
  .settings(
    name := "scala-sbt",
    version := "0.1.0-SNAPSHOT",

    scalaVersion := scala3Version,

    // This is required by nixpacks
    executableScriptName := "main"
  )
  // sbt-native-packager is the tool used by nixpacks
  // to generate the package
  .enablePlugins(JavaAppPackaging)

```

The `executableScriptName` is required for `nixpacks` to run the right executable
in the start command.

### Build

`sbt-native-package` is used for building the package.

```
sbt stage
```

This creates the required packages and also a convenient script to run
at `./target/universal/stage/bin/main`

### Start

Run the built script:

```sh
./target/universal/stage/bin/main
```

The script picks up `JAVA_OPTS` to provide jvm or java arguments to the system.
