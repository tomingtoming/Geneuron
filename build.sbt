import sbt._

name := "geneuron"

version := "1.0"

scalaVersion := "2.11.6"

libraryDependencies  ++= Seq(
  "org.slick2d"   % "slick2d-core"   % "1.0.1",
  "org.scalanlp" %% "breeze"         % "0.11.2",
  "org.scalanlp" %% "breeze-natives" % "0.11.2"
)

resolvers ++= Seq(
  "Sonatype Snapshots" at "https://oss.sonatype.org/content/repositories/snapshots/",
  "Sonatype Releases"  at "https://oss.sonatype.org/content/repositories/releases/"
)

fork in run := true

val os = System.getProperty("os.name").split(" ")(0).toLowerCase match {
  case "linux" => "linux"
  case "mac" => "macosx"
  case "windows" => "windows"
  case "sunos" => "solaris"
  case x => x
}

javaOptions in run += "-Djava.library.path=" + System.getProperty("java.library.path") + ":lib/slick"
