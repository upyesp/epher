package io.epher.jetbrains

import com.intellij.openapi.application.PathManager
import com.intellij.openapi.diagnostic.Logger
import org.jetbrains.plugins.textmate.api.TextMateBundleProvider
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Serves the shared epher grammar to the TextMate plugin.
 *
 * Registered under com.intellij.textmate.bundleProvider (verified in
 * the 2024.2 platform sources). The bundle travels in VS Code layout
 * (package.json, syntaxes/epher.tmLanguage, language-configuration.json),
 * which is the one form the TextMate plugin reads directly. Files with
 * no other file type, *.epher included, are picked up by the plugin's
 * own file type detector, so no custom FileType is contributed here.
 *
 * The TextMate plugin wants a real directory on disk, so the bundle is
 * unpacked from the plugin jar into the IDE's system directory once per
 * IDE run; a few small text files, cheap to rewrite.
 */
class EpherTextMateBundleProvider : TextMateBundleProvider {

    private val refreshed = AtomicBoolean(false)

    override fun getBundles(): List<TextMateBundleProvider.PluginBundle> =
        listOf(TextMateBundleProvider.PluginBundle(BUNDLE_NAME, bundleDirectory()))

    private fun bundleDirectory(): Path {
        val dir = Path.of(PathManager.getSystemPath(), SYSTEM_DIR, BUNDLE_NAME)
        if (refreshed.compareAndSet(false, true)) {
            for (relative in BUNDLE_FILES) {
                val target = dir.resolve(relative)
                Files.createDirectories(target.parent)
                val resource = "$RESOURCE_ROOT/$relative"
                val stream = javaClass.classLoader.getResourceAsStream(resource)
                    ?: throw IllegalStateException("missing plugin resource: $resource")
                stream.use { body ->
                    Files.copy(body, target, StandardCopyOption.REPLACE_EXISTING)
                }
            }
            LOG.info("epher TextMate bundle unpacked to $dir")
        }
        return dir
    }

    companion object {
        private val LOG = Logger.getInstance(EpherTextMateBundleProvider::class.java)

        private const val SYSTEM_DIR = "epher-jetbrains"
        private const val BUNDLE_NAME = "epher"
        private const val RESOURCE_ROOT = "epher-bundle"

        /** The full bundle, matching package.json's references. */
        private val BUNDLE_FILES = listOf(
            "package.json",
            "language-configuration.json",
            "syntaxes/epher.tmLanguage",
        )
    }
}
