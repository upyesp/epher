package io.epher.jetbrains

import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Key
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.openapi.wm.ToolWindowManager
import com.intellij.ui.JBColor
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.jcef.JBCefApp
import com.intellij.ui.jcef.JBCefBrowser
import java.awt.BorderLayout
import java.awt.Desktop
import java.io.File
import javax.swing.JEditorPane
import javax.swing.JPanel
import javax.swing.event.HyperlinkEvent

/**
 * The results pane (ADR-0069, decision 3): the JetBrains shape of the
 * VS Code webview, as far as the toolkits allow. One reusable panel
 * holds a JBCefBrowser rendering the run's report — every answer and
 * every error as rows anchored to their lines, every plot inlined as
 * SVG — and each run replaces the HTML, the way the VS Code pane does.
 *
 * Where the platform has no embedded browser (JCEF is a runtime
 * dependency IDE installs can lack), the panel degrades to a plain
 * Swing editor pane: text rows, and each graph written to a temporary
 * SVG file listed as a clickable line for the system viewer.
 */
class EpherResultsToolWindowFactory : ToolWindowFactory {

    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = EpherResultsPanel()
        val content = toolWindow.contentManager.factory.createContent(panel, "", false)
        content.putUserData(PANEL_KEY, panel)
        // The browser is a Disposable: tie it to the content so the
        // JCEF process goes away with the tool window.
        content.setDisposer(panel)
        toolWindow.contentManager.addContent(content)
    }

    companion object {
        internal val PANEL_KEY: Key<EpherResultsPanel> = Key.create(EpherResultsPanel::class.java.name)
        const val TOOL_WINDOW_ID = "epher results"

        /**
         * Entry point for the run action, callable from any thread: the
         * window is activated (which lazily creates the content on its
         * very first show) and the panel's HTML replaced, all on the
         * EDT via the activate runnable.
         */
        fun show(project: Project, fileName: String, report: RunReport) {
            ApplicationManager.getApplication().invokeLater {
                val toolWindow = ToolWindowManager.getInstance(project).getToolWindow(TOOL_WINDOW_ID)
                    ?: return@invokeLater
                toolWindow.activate {
                    val content = toolWindow.contentManager.getContent(0) ?: return@activate
                    content.getUserData(PANEL_KEY)?.update(report)
                    content.displayName = "results — $fileName"
                }
            }
        }
    }
}

/**
 * The pane surface itself: the browser when JCEF is present, the Swing
 * fallback when not, decided once at creation.
 */
internal class EpherResultsPanel : JPanel(BorderLayout()), Disposable {

    // JBCefApp.isSupported() is the canonical availability probe; a
    // platform hardening change may still throw on construction, and
    // both roads fall back to Swing rather than break the run.
    private val browser: JBCefBrowser? = try {
        if (JBCefApp.isSupported()) JBCefBrowser() else null
    } catch (broken: Throwable) {
        null
    }

    private val fallback: JEditorPane? = if (browser == null) createFallbackPane() else null

    init {
        val browser = this.browser
        if (browser != null) {
            add(browser.component, BorderLayout.CENTER)
        } else {
            fallback?.let { add(JBScrollPane(it), BorderLayout.CENTER) }
        }
    }

    fun update(report: RunReport) {
        val browser = this.browser
        if (browser != null) {
            browser.loadHTML(reportHtml(report))
        } else {
            fallback?.text = fallbackHtml(report)
        }
    }

    override fun dispose() {
        browser?.dispose()
    }

    /** The text-only pane: rows as text, graphs as clickable temp files. */
    private fun createFallbackPane(): JEditorPane = JEditorPane("text/html", "").apply {
        isEditable = false
        addHyperlinkListener { event ->
            if (event.eventType == HyperlinkEvent.EventType.ACTIVATED) {
                event.url?.let { url ->
                    // The JDK's own opener: zero platform-API surface, so
                    // this fallback compiles and works on every build.
                    try {
                        Desktop.getDesktop().browse(url.toURI())
                    } catch (broken: Exception) {
                        // No desktop integration: leave the path visible in
                        // the pane for the user to open by hand.
                    }
                }
            }
        }
    }

    private fun fallbackHtml(report: RunReport): String {
        val c = palette()
        val rows = report.statements
            .filter { it.display != null }
            .joinToString("\n") { statement ->
                val cls = if (statement.error) "error" else "answer"
                "<div class=\"output\"><span class=\"$cls\">${escapeHtml(statement.display ?: "")}</span></div>"
            }
        val graphs = report.svgs.mapIndexed { index, svg ->
            // Swing cannot render SVG: write each plot to a temporary
            // file and let the system viewer open it, the same road the
            // text-first editors take.
            val file = runCatching { writeSvg(index, svg) }.getOrNull()
            if (file != null) {
                "<p><a href=\"${file.toURI()}\">graph ${index + 1} — ${escapeHtml(file.path)}</a></p>"
            } else {
                "<p>graph ${index + 1} — could not write a temporary file</p>"
            }
        }.joinToString("\n")
        val body = if (rows.isEmpty() && report.svgs.isEmpty()) "<p class=\"empty\">No output.</p>" else rows
        val graphSection = if (report.svgs.isNotEmpty()) "<h2>Graphs</h2>\n$graphs" else ""
        return """
            <!DOCTYPE html>
            <html>
            <head>
            <meta charset="utf-8" />
            <style>
              body { font-family: monospace; color: ${c.foreground}; background: ${c.background}; padding: 8px 14px; }
              h2 { font-size: 1.05em; margin: 14px 0 6px; }
              .output { padding: 2px 8px; white-space: pre-wrap; }
              .answer { color: ${c.answer}; }
              .error { color: ${c.error}; }
              .empty { color: ${c.muted}; font-style: italic; }
            </style>
            </head>
            <body>
            $body
            $graphSection
            </body>
            </html>
            """.trimIndent()
    }

    private fun writeSvg(index: Int, svg: String): File =
        File.createTempFile("epher-graph-${index + 1}-", ".svg").apply { writeText(svg) }

    // A port of results.ts reportHtml(), minus the script: only
    // statements with output are rows (the pane is the script's output,
    // not a re-reading of it), then a Graphs section, then the empty
    // state. The data-line attributes stay on the rows: click-to-reveal
    // needs a JS bridge into the IDE, a planned follow-up (see README).
    private fun reportHtml(report: RunReport): String {
        val c = palette()
        val rows = report.statements
            .filter { it.display != null }
            .joinToString("\n") { statement ->
                val answer = if (statement.error) {
                    "<span class=\"error\">${escapeHtml(statement.display ?: "error")}</span>"
                } else {
                    "<span class=\"answer\">${escapeHtml(statement.display ?: "")}</span>"
                }
                "<div class=\"output\" data-line=\"${statement.line}\">$answer</div>"
            }
        // The SVG documents the engine renders are self-contained and
        // generated (never user input); inline them and let CSS size
        // them to the pane.
        val graphs = report.svgs.joinToString("\n") { svg -> "<div class=\"graph\">$svg</div>" }
        val graphSection = if (report.svgs.isNotEmpty()) "<h2>Graphs</h2>\n$graphs" else ""
        val body = if (rows.isNotEmpty() || report.svgs.isNotEmpty()) rows else "<p class=\"empty\">No output.</p>"
        return """
            <!DOCTYPE html>
            <html>
            <head>
            <meta charset="utf-8" />
            <style>
              body { font-family: monospace; color: ${c.foreground}; background: ${c.background}; padding: 8px 14px; }
              h2 { font-size: 1.05em; margin: 14px 0 6px; }
              .output { padding: 2px 8px; white-space: pre-wrap; }
              .output:hover { background: ${c.hover}; }
              .answer { color: ${c.answer}; }
              .error { color: ${c.error}; }
              .graph { margin: 10px 0; }
              .graph svg { width: 100%; height: auto; background: #ffffff; border-radius: 4px; }
              .empty { color: ${c.muted}; font-style: italic; }
            </style>
            </head>
            <body>
            $body
            $graphSection
            </body>
            </html>
            """.trimIndent()
    }

    private data class Palette(
        val background: String,
        val foreground: String,
        val answer: String,
        val error: String,
        val muted: String,
        val hover: String,
    )

    // One palette per theme, inlined: JCEF knows nothing of the IDE's
    // theme variables, so the values are chosen here in Kotlin and
    // baked into the HTML.
    private fun palette(): Palette =
        if (JBColor.isBright()) {
            Palette("#ffffff", "#1e1e1e", "#0067c0", "#c00000", "#8a8a8a", "#eef2f7")
        } else {
            Palette("#1e1f22", "#dfe1e5", "#c8c8c8", "#f75464", "#9d9d9d", "#37373d")
        }

    private fun escapeHtml(text: String): String =
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
}
