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
import com.kitfox.svg.SVGDiagram
import com.kitfox.svg.SVGUniverse
import java.awt.BorderLayout
import java.awt.Component
import java.awt.Color
import java.awt.Dimension
import java.awt.Font
import java.awt.Graphics2D
import java.awt.RenderingHints
import java.awt.image.BufferedImage
import java.io.File
import java.io.StringReader
import javax.swing.BorderFactory
import javax.swing.BoxLayout
import javax.swing.ImageIcon
import javax.swing.JLabel
import javax.swing.JPanel
import javax.swing.JScrollPane
import javax.swing.SwingConstants

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

    // The Swing fallback: a vertical stack of rows and inline graph
    // images, rebuilt for each run. Graphs are rasterized with
    // svgSalamander — the sandbox makes the old temp-file links both
    // invisible and unopenable.
    private val fallbackStack = JPanel().apply {
        layout = BoxLayout(this, BoxLayout.Y_AXIS)
    }

    init {
        val browser = this.browser
        if (browser != null) {
            add(browser.component, BorderLayout.CENTER)
        } else {
            add(JBScrollPane(fallbackStack), BorderLayout.CENTER)
        }
    }

    fun update(report: RunReport) {
        val browser = this.browser
        if (browser != null) {
            browser.loadHTML(reportHtml(report))
        } else {
            rebuildFallback(report)
        }
    }

    override fun dispose() {
        browser?.dispose()
    }

    private fun rebuildFallback(report: RunReport) {
        fallbackStack.removeAll()
        val c = palette()
        val color: (String) -> Color = { Color.decode(it) }
        val outputs = report.statements.filter { it.display != null }
        if (outputs.isEmpty() && report.svgs.isEmpty()) {
            fallbackStack.add(paneLabel("No output.", color(c.muted), italic = true))
        }
        for (statement in outputs) {
            fallbackStack.add(
                paneLabel(
                    statement.display ?: "",
                    color(if (statement.error) c.error else c.answer),
                )
            )
        }
        if (report.svgs.isNotEmpty()) {
            fallbackStack.add(paneLabel("Graphs", color(c.foreground), bold = true))
            report.svgs.forEachIndexed { index, svg ->
                val image = runCatching { rasterize(svg, fallbackWidth()) }.getOrNull()
                if (image != null) {
                    val graph = JLabel(ImageIcon(image))
                    graph.alignmentX = Component.CENTER_ALIGNMENT
                    graph.border = BorderFactory.createEmptyBorder(6, 8, 6, 8)
                    fallbackStack.add(graph)
                } else {
                    val file = runCatching { writeSvg(index, svg) }.getOrNull()
                    fallbackStack.add(
                        paneLabel(
                            file?.path ?: "graph ${index + 1} — could not render",
                            color(c.muted),
                        )
                    )
                }
            }
        }
        fallbackStack.revalidate()
        fallbackStack.repaint()
    }

    /** The pane's usable width: wide enough to read, capped for huge windows. */
    private fun fallbackWidth(): Int = size.width.takeIf { it > 200 } ?: 720

    /** Render the engine's own generated SVG into a plain image. */
    private fun rasterize(svg: String, targetWidth: Int): java.awt.Image {
        val universe = SVGUniverse()
        val uri = universe.loadSVG(StringReader(svg), "graph")
        val diagram: SVGDiagram = universe.getDiagram(uri)
            ?: error("the SVG did not parse")
        val naturalWidth = diagram.width.toDouble()
        val naturalHeight = diagram.height.toDouble()
        val width = if (naturalWidth > 0) naturalWidth else 800.0
        val height = if (naturalHeight > 0) naturalHeight else 600.0
        val scale = targetWidth / width
        val image = BufferedImage(
            targetWidth,
            (height * scale).toInt().coerceAtLeast(1),
            BufferedImage.TYPE_INT_ARGB,
        )
        val g: Graphics2D = image.createGraphics()
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON)
        g.setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON)
        g.scale(scale, scale)
        diagram.render(g)
        g.dispose()
        return image
    }

    private fun paneLabel(text: String, color: Color, bold: Boolean = false, italic: Boolean = false): JLabel {
        val style = when {
            bold -> Font.BOLD
            italic -> Font.ITALIC
            else -> Font.PLAIN
        }
        return JLabel(text).apply {
            font = Font(Font.MONOSPACED, style, 13)
            foreground = color
            alignmentX = Component.LEFT_ALIGNMENT
            border = BorderFactory.createEmptyBorder(2, 14, 2, 8)
        }
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
