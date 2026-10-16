package io.epher.jetbrains

import com.intellij.openapi.vfs.VirtualFile
import java.io.IOException

/**
 * The URIs a run puts on the LSP wire (ADR-0069).
 *
 * VirtualFile.url is the IDE's own URL shape, not the LSP's. On Windows
 * it glues the drive letter straight after "file://" (file://C:/Users/
 * ...), and it copies the VFS path verbatim — which is the raw disk
 * spelling, not a percent-encoded one: a directory with a space or a
 * non-ASCII name (C:/Users/Pete my, C:/Users/José) rides along raw.
 * epher-lsp reads textDocument URIs with a strict RFC 3986 parser
 * (lsp-types 0.97's fluent-uri), and a raw space, an accented byte or a
 * backslash is a parse error there: the server exits before answering
 * epher/run, which the one-shot session reports as "epher-lsp died
 * mid-run". (The platform's own LSP client escapes before sending; the
 * one-shot conversation must do the same.)
 *
 * The disk path run through java.nio's toUri is always the canonical
 * shape servers expect — file:///C:/Users/Pete%20my/proj/eclipse.epher —
 * with the same spelling for the document and its root, which keeps the
 * server's didOpen/epher/run string matching intact.
 */
internal object EpherUris {

    /** The canonical LSP document uri for a local file. */
    fun documentUri(file: VirtualFile): String = uriOf(file)

    /**
     * The canonical LSP root uri: the file's directory, or the file
     * itself when it sits at a drive (or filesystem) root.
     */
    fun rootUri(file: VirtualFile): String =
        file.parent?.let { uriOf(it) } ?: uriOf(file)

    /**
     * toNioPath answers a local file's disk path; anything it refuses
     * (a jar entry, a light file) cannot be run anyway, but fall back
     * to the IDE url rather than crash a run that might still work.
     */
    private fun uriOf(file: VirtualFile): String =
        try {
            file.toNioPath().toUri().toString()
        } catch (broken: IOException) {
            file.url
        } catch (broken: UnsupportedOperationException) {
            file.url
        }
}
