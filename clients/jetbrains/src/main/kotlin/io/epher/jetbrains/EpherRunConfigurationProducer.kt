package io.epher.jetbrains

import com.intellij.execution.actions.ConfigurationContext
import com.intellij.execution.actions.RunConfigurationProducer
import com.intellij.openapi.util.Ref
import com.intellij.psi.PsiElement

/**
 * Teaches the platform that a .epher file in the editor is runnable
 * (ADR-0069): this is what turns the play icon on, offers "Run" in the
 * Run menu, and makes Ctrl+Shift+F10 create and run a configuration
 * named after the script. Contexts are matched on the virtual file
 * alone — TextMate files carry thin PSI, so nothing here may depend on
 * a PSI type. At 242 the file arrives through the context's location
 * (ConfigurationContext has no getVirtualFile member anymore).
 */
class EpherRunConfigurationProducer :
    RunConfigurationProducer<EpherRunConfiguration>(EpherRunConfigurationType().configurationFactories.first()) {

    override fun setupConfigurationFromContext(
        configuration: EpherRunConfiguration,
        context: ConfigurationContext,
        sourceElement: Ref<PsiElement>?,
    ): Boolean {
        val file = context.location?.virtualFile ?: return false
        if (!file.name.endsWith(".epher")) return false
        configuration.scriptPath = file.path
        configuration.name = file.nameWithoutExtension
        return true
    }

    override fun isConfigurationFromContext(
        configuration: EpherRunConfiguration,
        context: ConfigurationContext,
    ): Boolean {
        val file = context.location?.virtualFile ?: return false
        // The path is the configuration's whole identity, so it is the
        // only comparison that matters.
        return configuration.scriptPath == file.path
    }
}
