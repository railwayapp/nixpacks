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

