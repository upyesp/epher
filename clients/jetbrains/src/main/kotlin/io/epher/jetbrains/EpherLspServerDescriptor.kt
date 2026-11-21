package io.epher.jetbrains

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.KillableColoredProcessHandler
import com.intellij.execution.process.OSProcessHandler
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerDescriptor
import java.nio.file.Path
import java.util.concurrent.ExecutionException
import java.util.concurrent.Future
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException

/**
 * Describes the epher-lsp process to the platform's LSP client.
 *
 * Method shapes verified against the 2024.2 (242) platform classes:
 * the constructor is (project, presentableName, vararg roots), the one
 * abstract member is isSupportedFile, and both startServerProcess and
 * createCommandLine are open. (The getFilePaths() seen in newer docs
 * does not exist at 242.) The epher-lsp binary speaks LSP over stdio
 * with no arguments, so the command line is just the binary path.
 *
 * One server instance serves both kinds ADR-0088 brings: .epher
 * scripts and markdown documents carrying an epher fence. The gates
 * below mirror the provider's, and the didOpen language id keys the
 * server's markdown projection.
 */
class EpherLspServerDescriptor(project: Project, private val server: Future<Path>) :
    LspServerDescriptor(project, "epher") {

    override fun isSupportedFile(file: VirtualFile): Boolean = EpherMarkdown.isRunnable(file)

    /** The LSP language id, matching what the VS Code pilot reports. */
    override fun getLanguageId(file: VirtualFile): String = EpherMarkdown.languageId(file)

    override fun startServerProcess(): OSProcessHandler = KillableColoredProcessHandler(createCommandLine())

    override fun createCommandLine(): GeneralCommandLine = GeneralCommandLine(waitForServer().toString())

    /**
     * Joins the first-use download, which the provider started on a
     * background thread. Blocking is fine here: the platform reaches
     * this method from its server-start coroutine, never the EDT. A
     * failed download surfaces as a clean IllegalStateException, which
     * the LSP machinery logs and reports through the server widget.
     */
    private fun waitForServer(): Path = try {
        server.get(DOWNLOAD_TIMEOUT_MINUTES, TimeUnit.MINUTES)
    } catch (e: ExecutionException) {
        throw IllegalStateException("epher-lsp is not available: ${e.cause?.message}", e.cause)
    } catch (e: TimeoutException) {
        throw IllegalStateException("timed out waiting for the epher-lsp download to finish")
    } catch (e: InterruptedException) {
        Thread.currentThread().interrupt()
        throw IllegalStateException("interrupted while waiting for the epher-lsp download", e)
    }

    private companion object {
        const val DOWNLOAD_TIMEOUT_MINUTES = 5L
    }
}
