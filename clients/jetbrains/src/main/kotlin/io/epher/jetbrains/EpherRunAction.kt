package io.epher.jetbrains

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.ReadAction
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import java.util.concurrent.TimeUnit

/**
 * Runs the whole script (ADR-0069): the JetBrains shape of the VS Code
 * run road. The buffer's text is grabbed under a read action, sent
 * through a one-shot epher-lsp session (EpherOneShot) on a background
 * thread, and the report lands in the results pane. Failures come back
 * as a balloon notification, not a silent log entry.
 */
class EpherRunAction : AnAction() {

    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    override fun update(event: AnActionEvent) {
        val file = event.getData(CommonDataKeys.VIRTUAL_FILE)
        event.presentation.isEnabledAndVisible =
            event.project != null && file != null && file.name.endsWith(".epher")
    }

    override fun actionPerformed(event: AnActionEvent) {
        val project = event.project ?: return
        val file = event.getData(CommonDataKeys.VIRTUAL_FILE) ?: return
        val target = readTarget(file) ?: run {
            notify(project, "could not read ${file.name}; is it still on disk?")
            return
        }
        // Task.Backgroundable runs on a pooled thread like the
        // downloader's supplyAsync does, and gives a long evaluation a
        // progress entry in the status bar for free.
        object : Task.Backgroundable(project, "Running epher script") {
            override fun run(indicator: ProgressIndicator) {
                indicator.text = "evaluating ${target.fileName}"
                val binary = try {
                    EpherServerDownloader.ensureDownloadedAsync()
                        .get(DOWNLOAD_TIMEOUT_MINUTES, TimeUnit.MINUTES)
                } catch (e: InterruptedException) {
                    Thread.currentThread().interrupt()
                    throw IllegalStateException("interrupted while waiting for the epher-lsp download", e)
                } catch (e: Exception) {
                    throw IllegalStateException(
                        "the epher language server is not available: ${rootMessage(e)}", e
                    )
                }
                val report = EpherOneShot.run(binary, target.documentUri, target.rootUri, target.text)
                ApplicationManager.getApplication().invokeLater {
                    EpherResultsToolWindowFactory.show(project, target.fileName, report)
                }
            }

            override fun onThrowable(error: Throwable) {
                notify(project, rootMessage(error))
            }
        }.queue()
    }

    /** The buffer's text and the file's URIs, read on the EDT safely. */
    private fun readTarget(file: VirtualFile): RunTarget? =
        ReadAction.compute<RunTarget?, RuntimeException> {
            val document = FileDocumentManager.getInstance().getDocument(file) ?: return@compute null
            RunTarget(
                documentUri = file.url,
                rootUri = file.parent?.url ?: file.url,
                text = document.text,
                fileName = file.name,
            )
        }

    /** The first message in the cause chain that says something useful. */
    private fun rootMessage(error: Throwable): String {
        var cause: Throwable = error
        while (cause.message.isNullOrBlank() && cause.cause != null && cause.cause !== cause) {
            cause = cause.cause!!
        }
        return cause.message ?: error.javaClass.simpleName
    }

    private fun notify(project: Project, message: String) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup(NOTIFICATION_GROUP_ID)
            .createNotification("epher run failed", message, NotificationType.ERROR)
            .notify(project)
    }

    private data class RunTarget(
        val documentUri: String,
        val rootUri: String,
        val text: String,
        val fileName: String,
    )

    private companion object {
        const val NOTIFICATION_GROUP_ID = "epher"
        const val DOWNLOAD_TIMEOUT_MINUTES = 5L
    }
}
