package io.epher.jetbrains

import com.intellij.execution.ui.ConsoleView
import com.intellij.execution.DefaultExecutionResult
import com.intellij.execution.runners.ExecutionEnvironment
import com.intellij.execution.ExecutionResult
import com.intellij.execution.Executor
import com.intellij.execution.configurations.RunProfileState
import com.intellij.execution.configurations.ConfigurationFactory
import com.intellij.execution.configurations.LocatableConfiguration
import com.intellij.execution.configurations.RunConfiguration
import com.intellij.execution.configurations.RunConfigurationBase
import com.intellij.execution.configurations.RuntimeConfigurationError
import com.intellij.execution.impl.ConsoleViewImpl
import com.intellij.execution.process.ProcessHandler
import com.intellij.execution.runners.ProgramRunner
import com.intellij.execution.ui.ConsoleViewContentType
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.ReadAction
import com.intellij.openapi.fileChooser.FileChooserDescriptorFactory
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.options.SettingsEditor
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.TextFieldWithBrowseButton
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.ui.LabeledComponent
import org.jdom.Element
import java.awt.BorderLayout
import java.io.File
import java.io.IOException
import java.io.OutputStream
import java.util.concurrent.TimeUnit
import javax.swing.JComponent
import javax.swing.JPanel

/**
 * The native run configuration (ADR-0069, decision 3: JetBrains takes
 * the VS Code shape where the toolkit allows). One persisted field —
 * the script path — and a run that drives the same one-shot epher-lsp
 * session as the Tools action (EpherOneShot), printed as a transcript
 * into a run tab's console. Graphs are not drawn here: the transcript
 * names the epher results pane for them, which the Tools action opens.
 *
 * Persistence rides the oldest, simplest road there is: readExternal
 * and writeExternal on a single attribute, the shape plugins have used
 * since before the options-bean machinery existed — nothing here to
 * break between platform versions.
 */
class EpherRunConfiguration(project: Project, factory: ConfigurationFactory, name: String? = null) :
    RunConfigurationBase<Void>(project, factory, name), LocatableConfiguration {

    /** The .epher file to run, as an absolute path on disk. */
    var scriptPath: String = ""

    override fun getState(executor: Executor, environment: ExecutionEnvironment): RunProfileState =
        EpherRunProfileState(environment, this)

    override fun getConfigurationEditor(): SettingsEditor<out RunConfiguration> = EpherSettingsEditor()

    override fun checkConfiguration() {
        if (scriptPath.isBlank()) throw RuntimeConfigurationError("no script chosen")
        if (!File(scriptPath).isFile) throw RuntimeConfigurationError("no such file: $scriptPath")
    }

    // LocatableConfiguration: the platform renames an auto-created
    // configuration to the suggested name after it first runs, and a
    // hand-renamed one stops counting as generated.
    override fun getSuggestedName(): String =
        File(scriptPath).nameWithoutExtension.ifEmpty { "epher" }

    override fun isGeneratedName(): Boolean = name == getSuggestedName()

    override fun writeExternal(element: Element) {
        super.writeExternal(element)
        if (scriptPath.isNotBlank()) element.setAttribute(SCRIPT_PATH_ATTRIBUTE, scriptPath)
    }

    override fun readExternal(element: Element) {
        super.readExternal(element)
        scriptPath = element.getAttributeValue(SCRIPT_PATH_ATTRIBUTE) ?: ""
    }

    private companion object {
        const val SCRIPT_PATH_ATTRIBUTE = "scriptPath"
    }
}

/**
 * The configuration's editor: one script-path field with the
 * platform's file browser. Producer-created configurations arrive with
 * the path already filled; the field exists so a hand-made
 * configuration can be pointed at a file too.
 */
private class EpherSettingsEditor : SettingsEditor<EpherRunConfiguration>() {

    private val path = TextFieldWithBrowseButton()

    init {
        path.addBrowseFolderListener(
            "epher script",
            "choose the .epher file to run",
            null,
            FileChooserDescriptorFactory.createSingleFileNoJarsDescriptor(),
        )
    }

    override fun createEditor(): JComponent =
        JPanel(BorderLayout()).apply {
            add(LabeledComponent.create(path, "Script path"), BorderLayout.NORTH)
        }

    override fun resetEditorFrom(configuration: EpherRunConfiguration) {
        path.text = configuration.scriptPath
    }

    override fun applyEditorTo(configuration: EpherRunConfiguration) {
        configuration.scriptPath = path.text.trim()
    }
}

/**
 * The run itself (ADR-0069). The console transcript is attached to a
 * placeholder process handler so the platform shows a proper run tab,
 * while the blocking epher-lsp conversation happens on a pooled thread
 * — EpherOneShot enforces its own 30-second deadline — and each report
 * line lands in the console as it is produced. A failure prints in red
 * and the tab ends with a non-zero exit code.
 */
private class EpherRunProfileState(
    private val environment: ExecutionEnvironment,
    private val configuration: EpherRunConfiguration,
) : RunProfileState {

    override fun execute(executor: Executor, runner: ProgramRunner<*>): ExecutionResult {
        val console = ConsoleViewImpl(environment.project, true)
        val handler = TranscriptHandler()
        handler.startNotify()
        // The conversation blocks for up to the one-shot deadline, so
        // it must not run on the thread that called execute, whatever
        // the platform chooses that to be.
        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                run(console)
                handler.finish(0)
            } catch (broken: Throwable) {
                console.print("${rootMessage(broken)}\n", ConsoleViewContentType.ERROR_OUTPUT)
                handler.finish(1)
            }
        }
        return DefaultExecutionResult(console, handler)
    }

    /** The blocking conversation; call off the EDT. */
    private fun run(console: ConsoleView) {
        val path = configuration.scriptPath
        val file = LocalFileSystem.getInstance().findFileByPath(path)
            ?: throw IOException("no such file: $path")
        // The buffer, not the disk file, is what a run should see: the
        // same read the Tools action does.
        val text = ReadAction.compute<String?, RuntimeException> {
            FileDocumentManager.getInstance().getDocument(file)?.text
        } ?: throw IOException("could not read $path")

        console.print("running ${file.name}\n", ConsoleViewContentType.SYSTEM_OUTPUT)
        val binary = try {
            EpherServerDownloader.ensureDownloadedAsync()
                .get(DOWNLOAD_TIMEOUT_MINUTES, TimeUnit.MINUTES)
        } catch (e: InterruptedException) {
            Thread.currentThread().interrupt()
            throw IOException("interrupted while waiting for the epher-lsp download", e)
        } catch (e: Exception) {
            throw IOException("the epher language server is not available: ${rootMessage(e)}", e)
        }
        val report = EpherOneShot.run(binary, file.url, file.parent?.url ?: file.url, text)
        for (statement in report.statements) {
            val display = statement.display ?: continue
            val kind = if (statement.error) "error" else "answer"
            val type = if (statement.error) {
                ConsoleViewContentType.ERROR_OUTPUT
            } else {
                ConsoleViewContentType.NORMAL_OUTPUT
            }
            console.print("line ${statement.line + 1} — $kind: $display\n", type)
        }
        if (report.svgs.isNotEmpty()) {
            // The console draws no graphs: point at the pane that does,
            // where the unchanged Tools action still shows them.
            val count = report.svgs.size
            console.print(
                "\n$count graph${if (count == 1) "" else "s"} rendered — " +
                    "Tools ▸ Run Epher Script shows them in the epher results pane\n",
                ConsoleViewContentType.SYSTEM_OUTPUT,
            )
        } else if (report.statements.none { it.display != null }) {
            console.print("no output\n", ConsoleViewContentType.SYSTEM_OUTPUT)
        }
    }

    /** The first message in the cause chain that says something useful. */
    private fun rootMessage(error: Throwable): String {
        var cause: Throwable = error
        while (cause.message.isNullOrBlank() && cause.cause != null && cause.cause !== cause) {
            cause = cause.cause!!
        }
        return cause.message ?: error.javaClass.simpleName
    }

    private companion object {
        const val DOWNLOAD_TIMEOUT_MINUTES = 5L
    }
}

/**
 * A process handler with no process behind it: EpherOneShot owns the
 * real epher-lsp process and tears it down itself. This one exists so
 * the run tab has the standard life cycle — started, then terminated —
 * and closes cleanly when the transcript is done.
 */
private class TranscriptHandler : ProcessHandler() {

    override fun destroyProcessImpl() {}

    override fun detachProcessImpl() {}

    override fun detachIsDefault(): Boolean = false

    override fun getProcessInput(): OutputStream? = null

    fun finish(exitCode: Int) {
        notifyProcessTerminated(exitCode)
    }
}
