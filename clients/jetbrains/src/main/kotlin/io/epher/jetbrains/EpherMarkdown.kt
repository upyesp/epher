package io.epher.jetbrains

import com.intellij.openapi.vfs.VirtualFile

/**
 * The markdown half of ADR-0088: epher code fences inside a markdown
 * document are a script. The language server owns the understanding:
 * it scans the client's text for `epher`-tagged fences, projects them
 * to one virtual script, and serves every feature in markdown
 * coordinates (crates/lsp/src/markdown.rs). This file only answers
 * the activation question the server cannot answer itself: should the
 * epher tooling attach to this file at all?
 *
 * `.epher` files always do. A `.md`/`.markdown` file does when it
 * carries at least one epher fence with content, so prose-only
 * markdown never sees a run action or a server spawn.
 *
 * The fence grammar here mirrors the server's exactly (a deliberate
 * copy, not a dependency): up to three leading spaces, three or more
 * of one fence character (backtick or tilde), the info string's first
 * word equal to `epher` case-insensitively, a backtick info string
 * may not contain a backtick, a closing fence is the same character
 * at least as long with nothing else on the line, and an unclosed
 * fence runs to end of file. Only the *does an epher block exist*
 * answer is needed, so no offsets are kept.
 */
object EpherMarkdown {

    /** Refuse to scan pathologically large files; the scan runs on the
     *  EDT when the platform asks whether a file is supported, and a
     *  real markdown document is kilobytes. A ten-megabyte file is
     *  not a script someone wants to run anyway. */
    private const val MAX_SCAN_BYTES = 10L * 1024 * 1024

    /** Whether the epher tooling (LSP, run, configurations) attaches. */
    fun isRunnable(file: VirtualFile): Boolean {
        if (file.name.endsWith(".epher")) return true
        if (!isMarkdown(file)) return false
        return hasEpherFence(file)
    }

    /** The LSP language id the server keys the file's kind on
     *  (the server treats languageId "markdown" as a projection). */
    fun languageId(file: VirtualFile): String =
        if (isMarkdown(file)) "markdown" else "epher"

    fun isMarkdown(file: VirtualFile): Boolean =
        file.name.endsWith(".md") || file.name.endsWith(".markdown")

    /** The fence scan over the file's text; unreadable or oversized
     *  files simply do not activate. */
    private fun hasEpherFence(file: VirtualFile): Boolean {
        if (file.length > MAX_SCAN_BYTES) return false
        val text = runCatching { file.text }.getOrNull() ?: return false
        return hasEpherFence(text)
    }

    /** Whether the text carries one epher fence with at least one
     *  content line, the smallest projection the server would build.
     *  Fence lines inside another (non-epher) fence are content, not
     *  fence lines, so the walk tracks the enclosing fence; and an
     *  epher fence never closed runs its content to end of file. */
    fun hasEpherFence(text: String): Boolean {
        var fenceChar: Char? = null
        var fenceLen = 0
        var inEpher = false
        var epherHasContent = false
        for (line in text.lineSequence()) {
            if (fenceChar == null) {
                val open = fenceOpen(line)
                if (open != null) {
                    fenceChar = open.char
                    fenceLen = open.length
                    inEpher = firstWord(open.info).equals("epher", ignoreCase = true)
                    epherHasContent = false
                }
            } else if (isClose(line, fenceChar, fenceLen)) {
                if (inEpher && epherHasContent) return true
                fenceChar = null
                fenceLen = 0
                inEpher = false
            } else if (inEpher) {
                epherHasContent = true
            }
        }
        return inEpher && epherHasContent
    }

    private fun fenceOpen(line: String): Open? {
        val indent = line.length - line.trimStart(' ').length
        if (indent > 3) return null
        val stripped = line.substring(indent)
        val fenceChar = stripped.firstOrNull() ?: return null
        if (fenceChar != '`' && fenceChar != '~') return null
        val runLen = stripped.takeWhile { it == fenceChar }.length
        if (runLen < 3) return null
        val info = stripped.substring(runLen)
        if (fenceChar == '`' && info.contains('`')) return null
        return Open(fenceChar, runLen, info)
    }

    private fun isClose(line: String, fenceChar: Char, fenceLen: Int): Boolean {
        val stripped = line.trimStart(' ')
        val runLen = stripped.takeWhile { it == fenceChar }.length
        return runLen >= fenceLen && stripped.substring(runLen).trim().isEmpty()
    }

    private fun firstWord(info: String): String =
        info.trim().split(Regex("\\s+")).firstOrNull() ?: ""

    private data class Open(val char: Char, val length: Int, val info: String)
}
