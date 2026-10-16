package io.epher.jetbrains

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.LspServerSupportProvider.LspServerStarter

/**
 * Presents the epher language to the platform's built-in LSP client.
 *
 * Registered under com.intellij.platform.lsp.serverSupportProvider
 * (verified in the 2024.2 platform sources; there is no
 * com.intellij.modules.lsp module to depend on at 242). The platform
 * calls [fileOpened] for every file that opens, and this provider
 * answers only for the epher extension: it kicks off the first-use
 * download of the shared server and hands the platform a descriptor.
 */
class EpherLspServerSupportProvider : LspServerSupportProvider {

    override fun fileOpened(project: Project, file: VirtualFile, starter: LspServerStarter) {
        if (!file.name.endsWith(".epher")) return
        // The download itself runs off the EDT (see the downloader); the
        // descriptor only joins the result when the platform starts the
        // process, which happens on a background coroutine.
        starter.ensureServerStarted(EpherLspServerDescriptor(project, EpherServerDownloader.ensureDownloadedAsync()))
    }
}
