package io.epher.jetbrains

import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.application.PathManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.extensions.PluginId
import com.intellij.openapi.util.SystemInfo
import java.io.IOException
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.attribute.PosixFilePermissions
import java.time.Duration
import java.util.concurrent.CompletableFuture
import java.util.concurrent.ConcurrentHashMap
import java.util.zip.GZIPInputStream
import java.util.zip.ZipInputStream

/**
 * Downloads and caches the shared epher-lsp binary for this platform.
 *
 * Mirrors clients/vscode/src/download.ts (ADR-0066): the server is
 * never searched on PATH, it is fetched once from the GitHub release
 * matching the plugin's own version
 * (releases/download/v<version>/epher-lsp-<target>), unpacked into the
 * IDE's system directory (epher/bin), and marked with the version so
 * re-downloads only happen when the plugin updates. Linux and macOS
 * assets are gzipped binaries, the Windows asset is a zip with the exe
 * at the archive root.
 */
object EpherServerDownloader {

    private val LOG = Logger.getInstance(EpherServerDownloader::class.java)

    private const val REPO = "upyesp/epher"
    const val PLUGIN_ID = "io.epher.jetbrains"

    /** Fallback when the descriptor lookup fails; must equal the build default. */
    private const val DEFAULT_VERSION = "0.5.40"

    private const val EXE_NAME = "epher-lsp"
    private const val EXE_NAME_WINDOWS = "epher-lsp.exe"
    private const val MARKER_NAME = "server-version"
    private const val USER_AGENT = "epher-jetbrains"

    /** The download for each requested version, started at most once. */
    private val downloads = ConcurrentHashMap<String, CompletableFuture<Path>>()

    /**
     * Returns the future holding the cached server path, starting the
     * download the first time a version is requested. Safe to call on
     * the EDT: the network work always runs on a background thread and
     * the descriptor joins the future later, off the EDT.
     */
    fun ensureDownloadedAsync(): CompletableFuture<Path> {
        val version = pluginVersion()
        return downloads.computeIfAbsent(version) { v ->
            CompletableFuture.supplyAsync { ensureDownloaded(v) }
        }
    }

    private fun ensureDownloaded(version: String): Path {
        val binDir = Path.of(PathManager.getSystemPath(), "epher", "bin")
        val exe = binDir.resolve(if (SystemInfo.isWindows) EXE_NAME_WINDOWS else EXE_NAME)
        val marker = binDir.resolve(MARKER_NAME)
        if (Files.isRegularFile(exe) && Files.isRegularFile(marker) && Files.readString(marker).trim() == version) {
            LOG.info("epher-lsp $version already cached: $exe")
            return exe
        }

        val asset = assetName()
        val url = "https://github.com/$REPO/releases/download/v$version/$asset"
        LOG.info("downloading $url")
        val archive = download(url)
        Files.createDirectories(binDir)
        if (asset.endsWith(".gz")) {
            GZIPInputStream(archive.inputStream()).use { gunzipped ->
                Files.copy(gunzipped, exe, StandardCopyOption.REPLACE_EXISTING)
            }
        } else {
            unzipRootEntry(archive, exe, EXE_NAME_WINDOWS)
        }
        makeExecutable(exe)
        // The marker goes down last: its presence means the exe is whole.
        Files.writeString(marker, version)
        LOG.info("epher-lsp $version ready: $exe")
        return exe
    }

    /**
     * The release asset for this machine. ADR-0066 ships four targets;
     * anything else (Intel macOS, Windows ARM64) arrives later, and the
     * error says so instead of guessing.
     */
    private fun assetName(): String {
        val os = when {
            SystemInfo.isLinux -> "linux"
            SystemInfo.isMac -> "macos"
            SystemInfo.isWindows -> "windows"
            else -> throw IllegalStateException(
                "no epher-lsp build for ${System.getProperty("os.name")}; " +
                    "the plugin ships for linux-x86_64, linux-aarch64, macos-aarch64, windows-x86_64"
            )
        }
        val arch = when (System.getProperty("os.arch")) {
            "amd64", "x86_64" -> "x86_64"
            "aarch64", "arm64" -> "aarch64"
            else -> throw IllegalStateException(
                "no epher-lsp build for ${System.getProperty("os.arch")}; " +
                    "the plugin ships for linux-x86_64, linux-aarch64, macos-aarch64, windows-x86_64"
            )
        }
        return "epher-lsp-$os-$arch.${if (SystemInfo.isWindows) "zip" else "gz"}"
    }

    /**
     * GET with redirect following (GitHub's release assets answer 302)
     * and a clean, explainable error for every failure mode.
     */
    private fun download(url: String): ByteArray {
        val client = HttpClient.newBuilder()
            .followRedirects(HttpClient.Redirect.NORMAL)
            .connectTimeout(Duration.ofSeconds(15))
            .build()
        val request = HttpRequest.newBuilder(URI.create(url))
            .header("User-Agent", USER_AGENT)
            .timeout(Duration.ofMinutes(5))
            .GET()
            .build()
        val response = try {
            client.send(request, HttpResponse.BodyHandlers.ofByteArray())
        } catch (e: IOException) {
            throw IOException("could not reach $url: ${e.message}", e)
        }
        if (response.statusCode() != 200) {
            val hint = if (response.statusCode() == 404) {
                " (the release has no such asset yet; language-server assets ride the promoted releases)"
            } else {
                ""
            }
            throw IOException("$url answered ${response.statusCode()}$hint")
        }
        return response.body()
    }

    /** The Windows zip carries the exe at the archive root, nothing else. */
    private fun unzipRootEntry(archive: ByteArray, target: Path, entryName: String) {
        ZipInputStream(archive.inputStream()).use { zip ->
            var entry = zip.nextEntry
            while (entry != null) {
                if (entry.name == entryName || entry.name.endsWith("/$entryName")) {
                    Files.copy(zip, target, StandardCopyOption.REPLACE_EXISTING)
                    return
                }
                entry = zip.nextEntry
            }
        }
        throw IOException("the Windows asset has no $entryName at the archive root")
    }

    /** The downloaded binary must be runnable; zip/gzip lose the mode. */
    private fun makeExecutable(exe: Path) {
        if (SystemInfo.isWindows) return
        Files.setPosixFilePermissions(exe, PosixFilePermissions.fromString("rwxr-xr-x"))
    }

    /**
     * The plugin's own version pins the server's (ADR-0066: an
     * extension update re-fetches a matching server).
     */
    private fun pluginVersion(): String =
        PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID))?.version
            ?.takeUnless { it.isBlank() }
            ?: DEFAULT_VERSION
}
