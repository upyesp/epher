package io.epher.jetbrains

import com.google.gson.JsonElement
import com.google.gson.JsonNull
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.google.gson.JsonSyntaxException
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit

/**
 * One statement result from `epher/run` (ADR-0069). [line] is the
 * 0-based source line the statement starts on, [source] the statement
 * text, [display] what the statement printed (null when it printed
 * nothing), and [error] marks a failed statement.
 */
data class RunLine(val line: Int, val source: String, val display: String?, val error: Boolean)

/**
 * A finished run: every statement's line plus the plots the run
 * produced, ready for the results pane.
 */
data class RunReport(val statements: List<RunLine>, val svgs: List<String>)

/**
 * A one-shot script run over a short-lived epher-lsp session
 * (ADR-0069, decision 3: JetBrains takes the VS Code shape where the
 * UI toolkits allow it).
 *
 * The platform's built-in LSP client owns the inline-hints server
 * process and exposes no raw LanguageServer handle, so a run speaks to
 * its own epher-lsp over stdio: plain JSON-RPC with Content-Length
 * framing, exactly the language-server wire format. The binary is
 * already on disk (EpherServerDownloader caches it for first use); the
 * run only borrows it. The session lives for the single run:
 * initialize, didOpen, `epher/run`, shutdown, exit. The run is
 * hermetic, the same fresh-session semantics the VS Code family has.
 */
object EpherOneShot {

    private const val TIMEOUT_SECONDS = 30L

    /**
     * Runs the script to completion on the calling thread; call from a
     * background thread only. Every failure comes back as an IOException
     * with a message meant for the user.
     */
    fun run(binary: Path, documentUri: String, rootUri: String, text: String): RunReport {
        if (!Files.isRegularFile(binary)) {
            throw IOException(
                "the epher-lsp binary is missing at $binary; " +
                    "reopen a .epher file to re-trigger the download"
            )
        }
        return Session(binary, documentUri, rootUri, text).converse()
    }

    private class Session(
        private val binary: Path,
        private val documentUri: String,
        private val rootUri: String,
        private val text: String,
    ) {
        // One deadline for the whole conversation: the run must finish
        // inside it or the process is torn down.
        private val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(TIMEOUT_SECONDS)

        // Incoming frames; JsonNull marks end of stream. A poll that
        // returns null is a timeout, never an EOF: those arrive as
        // JsonNull, so the two failure roads stay distinguishable.
        private val inbox = LinkedBlockingQueue<JsonElement>()

        // Why the reader thread gave up, when it was not a clean EOF.
        @Volatile
        private var readFailure: String? = null

        fun converse(): RunReport {
            val process = try {
                ProcessBuilder(binary.toString())
                    // Server logs (the "ready" line) go to stderr; inherit
                    // so they land in the IDE's own log instead of filling
                    // a pipe nobody drains.
                    .redirectError(ProcessBuilder.Redirect.INHERIT)
                    .start()
            } catch (e: IOException) {
                throw IOException("could not start epher-lsp ($binary): ${e.message}", e)
            }
            startReader(process)
            try {
                send(process.outputStream, request(INITIALIZE_ID, "initialize", JsonObject().apply {
                    add("processId", JsonNull.INSTANCE)
                    addProperty("rootUri", rootUri)
                    add("capabilities", JsonObject())
                }))
                expect(INITIALIZE_ID, "initialize")

                send(process.outputStream, notification("initialized", JsonObject()))
                // The run reads the document it was shown: didOpen hands
                // over the buffer exactly as it looked when Run fired.
                send(process.outputStream, notification("textDocument/didOpen", JsonObject().apply {
                    add("textDocument", JsonObject().apply {
                        addProperty("uri", documentUri)
                        addProperty("languageId", "epher")
                        addProperty("version", 1)
                        addProperty("text", text)
                    })
                }))

                val report = parseReport(expect(RUN_ID, "epher/run"))

                // The polite end of a life cycle: shutdown answers, then
                // exit ends the server loop. The server may already be
                // gone when exit is written; a broken pipe there is fine.
                send(process.outputStream, request(SHUTDOWN_ID, "shutdown", JsonNull.INSTANCE))
                expect(SHUTDOWN_ID, "shutdown")
                try {
                    send(process.outputStream, notification("exit", JsonObject()))
                } catch (broken: IOException) {
                }
                return report
            } finally {
                process.destroy()
                if (!process.waitFor(5, TimeUnit.SECONDS)) {
                    process.destroyForcibly()
                }
            }
        }

        /** Reads frames off the server's stdout into the inbox, forever. */
        private fun startReader(process: Process) {
            val reader = Thread({
                val input = process.inputStream
                while (true) {
                    val frame = try {
                        readFrame(input)
                    } catch (e: IOException) {
                        readFailure = e.message
                        JsonNull.INSTANCE
                    } catch (e: RuntimeException) {
                        readFailure = e.message
                        JsonNull.INSTANCE
                    }
                    inbox.put(frame)
                    if (frame.isJsonNull) return@Thread
                }
            }, "epher one-shot server reader")
            reader.isDaemon = true
            reader.start()
        }

        /**
         * The response to the given request id, skipping whatever
         * notifications pass by on the way (the server publishes
         * diagnostics between didOpen and the run's answer).
         */
        private fun expect(id: Int, what: String): JsonElement {
            while (true) {
                val remaining = deadline - System.nanoTime()
                if (remaining <= 0) throw timedOut(what)
                val message = inbox.poll(remaining, TimeUnit.NANOSECONDS) ?: throw timedOut(what)
                if (!message.isJsonObject) {
                    val why = readFailure?.let { "; $it" } ?: "the process ended before answering $what"
                    throw IOException("epher-lsp died mid-run$why")
                }
                // get(String) lives on JsonObject, not JsonElement; the
                // isJsonObject gate above is what makes this cast safe.
                val body = message.asJsonObject
                val messageId = body.get("id")
                // No id, or not ours: a notification or a stranger; keep reading.
                if (messageId == null || !messageId.isJsonPrimitive || messageId.asInt != id) continue
                body.get("error")?.takeIf { !it.isJsonNull }?.let { error ->
                    val detail = (error as? JsonObject)?.get("message")
                        ?.takeIf { it.isJsonPrimitive }?.asString ?: "no detail given"
                    throw IOException("epher-lsp failed $what: $detail")
                }
                return body.get("result") ?: JsonNull.INSTANCE
            }
        }

        private fun timedOut(what: String): IOException =
            IOException("timed out after $TIMEOUT_SECONDS seconds waiting for $what from epher-lsp")

        /** One framed JSON message, or JsonNull at end of stream. */
        private fun readFrame(input: InputStream): JsonElement {
            var contentLength = -1
            while (true) {
                val line = readHeaderLine(input) ?: return JsonNull.INSTANCE
                if (line.isEmpty()) break
                if (line.startsWith("Content-Length:", ignoreCase = true)) {
                    contentLength = line.substringAfter(':').trim().toIntOrNull() ?: -1
                }
            }
            if (contentLength < 0) {
                throw IOException("a frame from epher-lsp has no Content-Length header")
            }
            val body = input.readNBytes(contentLength)
            if (body.size < contentLength) return JsonNull.INSTANCE
            return try {
                JsonParser.parseString(body.toString(StandardCharsets.UTF_8))
            } catch (e: JsonSyntaxException) {
                throw IOException("could not parse a frame from epher-lsp: ${e.message}")
            }
        }

        /** One CRLF-terminated header line, or null at end of stream. */
        private fun readHeaderLine(input: InputStream): String? {
            val line = StringBuilder()
            while (true) {
                when (val b = input.read()) {
                    -1 -> return if (line.isEmpty()) null else line.toString()
                    '\n'.code -> return line.toString().removeSuffix("\r")
                    else -> line.append(b.toChar())
                }
            }
        }

        private fun send(output: OutputStream, message: JsonObject) {
            val body = message.toString().toByteArray(StandardCharsets.UTF_8)
            output.write("Content-Length: ${body.size}\r\n\r\n".toByteArray(StandardCharsets.US_ASCII))
            output.write(body)
            output.flush()
        }

        private fun request(id: Int, method: String, params: JsonElement): JsonObject = JsonObject().apply {
            addProperty("jsonrpc", "2.0")
            addProperty("id", id)
            addProperty("method", method)
            add("params", params)
        }

        private fun notification(method: String, params: JsonElement): JsonObject = JsonObject().apply {
            addProperty("jsonrpc", "2.0")
            addProperty("method", method)
            add("params", params)
        }

        private fun parseReport(result: JsonElement): RunReport =
            try {
                val body = result.asJsonObject
                val statements = body.get("statements")?.takeIf { it.isJsonArray }
                    ?.asJsonArray
                    ?.filter { it.isJsonObject }
                    ?.map { statement ->
                        val o = statement.asJsonObject
                        RunLine(
                            line = o.get("line")?.takeIf { it.isJsonPrimitive }?.asInt ?: 0,
                            source = o.get("source")?.takeIf { it.isJsonPrimitive }?.asString ?: "",
                            display = o.get("display")?.takeIf { it.isJsonPrimitive }?.asString,
                            error = o.get("error")?.takeIf { it.isJsonPrimitive }?.asBoolean ?: false,
                        )
                    }
                    ?: emptyList()
                val svgs = body.get("svgs")?.takeIf { it.isJsonArray }
                    ?.asJsonArray
                    ?.filter { it.isJsonPrimitive }
                    ?.map { it.asString }
                    ?: emptyList()
                RunReport(statements, svgs)
            } catch (e: IllegalStateException) {
                throw IOException("epher-lsp answered a run result this plugin cannot read: ${e.message}")
            }
    }

    // Request ids live at object level: a companion object cannot nest
    // inside a standalone object.
    private const val INITIALIZE_ID = 1
    private const val RUN_ID = 2
    private const val SHUTDOWN_ID = 3
}
