import java.io.File

// Build of the epher plugin for the JetBrains IDEs (ADR-0066, stage
// five). The plugin is a thin shell: the shared grammar rides as a
// TextMate bundle, the shared snippets ride as native live templates,
// and the platform's built-in LSP client is wired to the shared
// epher-lsp binary, which the plugin downloads on first use from the
// release the version pins.
//
// Toolchain facts verified against the platform sources before writing
// code (see the roadmap's stage five notes):
//   - the LSP API (com.intellij.platform.lsp.api) ships only in the
//     commercial IDEs; the IDEA Community 2024.2.4 artifact carries no
//     LSP classes, so this build depends on IDEA Ultimate 2024.2.4,
//   - at 242 there is no com.intellij.modules.lsp module to depend on;
//     the extension point com.intellij.platform.lsp.serverSupportProvider
//     lives in the IDE product itself,
//   - the TextMate plugin (org.intellij.plugins.textmate) is bundled
//     everywhere and exposes com.intellij.textmate.bundleProvider.

plugins {
    id("java")
    kotlin("jvm") version "2.2.20"
    id("org.jetbrains.intellij.platform") version "2.18.1"
}

// The plugin version rides the epher release train (0.5.x); CI passes
// -PpluginVersion so the packaged version always matches the CLI it
// will pair with.
group = "io.epher"
version = providers.gradleProperty("pluginVersion").getOrElse("0.5.40")

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        // Commercial IDE only: see the header comment. 2024.2.x is the
        // oldest line with the LSP API public, hence sinceBuild 242.
        intellijIdeaUltimate("2024.2.4")
    }
    // The bundledPlugin(...) helper cannot see the TextMate plugin in
    // this distribution: its Ide lookup misses entries until the
    // plugin list has been materialized once, and IPG's first call is
    // the lookup (plugin-structure lazy-init bug). The plugin is a
    // compile-time-only need (one interface); its jar is picked out of
    // the extracted IDE below, and at runtime the declared plugin
    // dependency in plugin.xml provides it.
    compileOnly(files(layout.buildDirectory.file("textmate/textmate.jar")))
    // The IDE ships the stdlib at runtime (see gradle.properties), the
    // compile classpath still needs it spelled out.
    compileOnly("org.jetbrains.kotlin:kotlin-stdlib:2.2.20")
}

// Extract the bundled TextMate plugin's jar from the resolved IDE into
// the build directory, so the compileOnly dependency above has a plain
// file to compile against. The IDE artifact resolves at execution time
// (it carries the IDE download and extraction with it).
val prepareTextmateJar = tasks.register("prepareTextmateJar") {
    val out = layout.buildDirectory.file("textmate/textmate.jar")
    outputs.file(out)
    doLast {
        val artifactFiles = configurations.getByName("intellijPlatformDependency").incoming.artifacts.artifactFiles
        val jar: File = artifactFiles.files
            .filter { it.isDirectory }
            .mapNotNull { dir -> File(dir, "plugins/textmate/lib/textmate.jar").takeIf { f -> f.exists() } }
            .firstOrNull() ?: error("the extracted IntelliJ Platform does not bundle the TextMate plugin jar")
        val target = out.get().asFile
        target.parentFile.mkdirs()
        jar.copyTo(target, overwrite = true)
    }
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
}

kotlin {
    jvmToolchain(21)
}

intellijPlatform {
    // Neither applies to this plugin: there is nothing to index for
    // searchable options and no Java sources to instrument, and both
    // save a noticeable slice of CI time.
    buildSearchableOptions = false
    instrumentCode = false

    pluginConfiguration {
        // 2024.2: the first line where the LSP API is public. No upper
        // bound: the extension points used here are platform-level and
        // the website distributes per-version zips.
        ideaVersion {
            sinceBuild = "242"
            untilBuild = provider { null }
        }
    }
}

tasks {
    compileKotlin {
        dependsOn(prepareTextmateJar)
    }
    compileJava {
        dependsOn(prepareTextmateJar)
    }
}
