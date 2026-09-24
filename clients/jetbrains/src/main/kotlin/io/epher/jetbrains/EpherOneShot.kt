package io.epher.jetbrains

import com.google.gson.JsonElement
import com.google.gson.JsonNull
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.google.gson.JsonSyntaxException
import com.intellij.execution.ExecutionException
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.diagnostic.Logger
import java.io.IOException
import java.io.InputStream
import java.io.InputStreamReader
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

    // One deadline for the whole conversation: the run must finish
    // inside it or the process is torn down. Ten minutes, measured
    // against the script collection itself: the server is synchronous,
    // so this deadline covers BOTH full evaluations of the document —
    // the didOpen analysis pass (didOpen is answered only after the
    // whole file has run) and the epher/run evaluation — and real
    // astronomy scripts need up to about two minutes for the two
    // together on a mid-range machine (mercury-transits.epher measured
    // 57 s analysis + 56 s run). VS Code, which shares this protocol,
    // waits without a timeout; the cap here exists only to reap a
    // wedged or crashed session, not to police the engine's compute.
    private const val TIMEOUT_SECONDS = 600L

    // Diagnostics for the field's "died mid-run" reports: the INFO line
    // here is the only record of what was sent, and the stderr tail the
    // only record of what the server said back. The IDE is launched
    // from a GUI, so the server's stderr has no console to land on —
    // without this capture the explanation dies with the process.
    private val LOG = Logger.getInstance("epher")

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
        // The one deadline for the whole conversation, set once here.
        private val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(TIMEOUT_SECONDS)

        // Incoming frames; JsonNull marks end of stream. A poll that
        // returns null is a timeout, never an EOF: those arrive as
        // JsonNull, so the two failure roads stay distinguishable.
        private val inbox = LinkedBlockingQueue<JsonElement>()

        // Why the reader thread gave up, when it was not a clean EOF.
        @Volatile
        private var readFailure: String? = null

        // Set by converse before any expect: the process and the
        // stderr tail the failure messages are built from.
        private var process: Process? = null
        private var tail: StderrTail = StderrTail()

        fun converse(): RunReport {
            // The exact conversation inputs, logged before anything
            // starts: a "died mid-run" report is only diagnosable with
            // the command line and the two URIs that went over the wire
            // (bad URI shapes have been the actual killer — EpherUris).
            LOG.info("epher one-shot run: command \"$binary\" documentUri=$documentUri rootUri=$rootUri")
            val tail = StderrTail()
            val process = try {
                // Spawn through the platform's GeneralCommandLine —
                // never a raw ProcessBuilder. The CONSOLE parent
                // environment rebuilds the child's environment from
                // the login shell, the shape every mature LSP plugin
                // spawns with (huggingface/llm-intellij, oxc) and the
                // same machinery the long-lived session uses — whose
                // server answers fine, where a raw-spawned one wedged
                // on a Flatpak IDE (0.5.50 field report: the server
                // consumed the whole conversation and never answered;
                // the identical conversation from python, inside the
                // same sandbox, answered instantly). stderr stays its
                // own pipe: the drain below needs it for failure
                // messages, as before.
                GeneralCommandLine(binary.toString())
                    .withParentEnvironmentType(GeneralCommandLine.ParentEnvironmentType.CONSOLE)
                    .withCharset(StandardCharsets.UTF_8)
                    .createProcess()
            } catch (e: ExecutionException) {
                throw IOException("could not start epher-lsp ($binary): ${e.message}", e)
            } catch (e: IOException) {
                throw IOException("could not start epher-lsp ($binary): ${e.message}", e)
            }
            this.process = process
            this.tail = tail
            startReader(process)
            startStderrDrain(process, tail)
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

                // The run request itself. Its absence was the field bug:
                // the loop below waited for a response to a request that
                // was never sent, so every run — one line or a thousand —
                // sat until the deadline on every platform. Found by
                // stracing the IDE during a live hang: initialize,
                // initialized and didOpen go out, then silence.
                send(process.outputStream, request(RUN_ID, "epher/run", JsonObject().apply {
                    add("textDocument", JsonObject().apply {
                        addProperty("uri", documentUri)
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
                    val why = readFailure?.let { "a read failed: $it" }
                        ?: "the process ended before answering $what"
                    throw died(why)
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

        /** The failure a dead server produces: its exit state plus its last words. */
        private fun died(why: String): IOException {
            val running = process
            val state = when {
                running == null -> ""
                running.isAlive -> "the process is still running"
                else -> "exit code ${running.exitValue()}"
            }
            val lastWords = tail.snapshot().trim()
            val message = "epher-lsp died mid-run; $why" +
                (if (state.isEmpty()) "" else " ($state)") +
                (if (lastWords.isEmpty()) ""
                 else "; it said: ${lastWords.take(MESSAGE_TAIL_CHARS)}")
            LOG.warn("$message [command=\"$binary\" documentUri=$documentUri rootUri=$rootUri]")
            return IOException(message)
        }

        private fun timedOut(what: String): IOException {
            val lastWords = tail.snapshot().trim()
            val message = "timed out after $TIMEOUT_SECONDS seconds waiting for $what from epher-lsp" +
                (if (lastWords.isEmpty()) ""
                 else "; it said: ${lastWords.take(MESSAGE_TAIL_CHARS)}")
            LOG.warn("$message [command=\"$binary\" documentUri=$documentUri rootUri=$rootUri]")
            return IOException(message)
        }

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

    /**
     * The server's last words: stderr is drained on a daemon thread
     * into this bounded buffer, because a GUI-launched IDE has no
     * console for the pipe to inherit and a wedged reader would stall
     * the server's own writes. Bounded so a chatty server cannot grow
     * the heap; only the tail is ever shown.
     */
    private class StderrTail(private val limit: Int = DEFAULT_LIMIT) {
        private val lock = Any()
        private var text = StringBuilder()

        fun append(chunk: CharArray, count: Int) {
            synchronized(lock) {
                text.append(chunk, 0, count)
                if (text.length > limit) text.delete(0, text.length - limit)
            }
        }

        fun snapshot(): String = synchronized(lock) { text.toString() }

        private companion object {
            const val DEFAULT_LIMIT = 8 * 1024
        }
    }

    /** Drains the server's stderr into the tail buffer until it closes. */
    private fun startStderrDrain(process: Process, tail: StderrTail) {
        val drain = Thread({
            val stream = InputStreamReader(process.errorStream, StandardCharsets.UTF_8)
            val chunk = CharArray(2048)
            while (true) {
                val count = try {
                    stream.read(chunk)
                } catch (broken: IOException) {
                    break
                }
                if (count < 0) break
                if (count > 0) tail.append(chunk, count)
            }
        }, "epher one-shot stderr drain")
        drain.isDaemon = true
        drain.start()
    }

    // Request ids live at object level: a companion object cannot nest
    // inside a standalone object.
    private const val INITIALIZE_ID = 1
    private const val RUN_ID = 2
    private const val SHUTDOWN_ID = 3

    /** How much of the server's stderr fits in a user-facing message. */
    private const val MESSAGE_TAIL_CHARS = 600
}
