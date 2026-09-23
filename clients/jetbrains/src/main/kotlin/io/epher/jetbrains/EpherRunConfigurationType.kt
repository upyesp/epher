package io.epher.jetbrains

import com.intellij.execution.configurations.ConfigurationFactory
import com.intellij.execution.configurations.ConfigurationType
import com.intellij.execution.configurations.RunConfiguration
import com.intellij.icons.AllIcons
import com.intellij.openapi.project.Project
import javax.swing.Icon

/**
 * The run-configuration type behind the play icon (ADR-0069): with it
 * registered, the platform knows a .epher file is runnable and stops
 * answering "the file in the editor is not runnable". The type is only
 * a shell around one factory producing EpherRunConfiguration; the
 * icon is a bundled platform constant, so no new images ship.
 */
class EpherRunConfigurationType : ConfigurationType {

    private val factory = object : ConfigurationFactory(this) {
        override fun createTemplateConfiguration(project: Project): RunConfiguration =
            EpherRunConfiguration(project, this)

        // The factory id rides the run manager's serialization key, so
        // it must be a stable, non-localized literal — the platform
        // deprecates the default, which delegates to getName (the type's
        // display name, localization bait). The value is "epher", the
        // exact string the default has always produced here
        // (type displayName = "epher"): already-shipped configurations
        // keep resolving, and there is no collision with the type id —
        // factory ids live inside their type's namespace.
        override fun getId(): String = FACTORY_ID
    }

    override fun getDisplayName(): String = "epher"

    override fun getConfigurationTypeDescription(): String = "run an epher script"

    override fun getIcon(): Icon = AllIcons.Nodes.Console

    override fun getId(): String = TYPE_ID

    override fun getConfigurationFactories(): Array<ConfigurationFactory> = arrayOf(factory)

    companion object {
        const val TYPE_ID = "epher"
        const val FACTORY_ID = "epher"
    }
}
