package io.epher.jetbrains

import com.intellij.execution.executors.DefaultRunExecutor
import com.intellij.execution.configurations.RunProfile
import com.intellij.execution.runners.DefaultProgramRunner

/**
 * The runner that executes EpherRunConfiguration's state (ADR-0069):
 * the standard DefaultProgramRunner shape, which calls the state's
 * execute and shows the returned console as a run tab. Run only —
 * debugging is a later phase (ADR-0069, decision 4), so the Debug
 * executor is declined here.
 */
class EpherProgramRunner : DefaultProgramRunner() {

    override fun getRunnerId(): String = RUNNER_ID

    override fun canRun(executorId: String, profile: RunProfile): Boolean =
        executorId == DefaultRunExecutor.EXECUTOR_ID && profile is EpherRunConfiguration

    private companion object {
        const val RUNNER_ID = "epher"
    }
}
