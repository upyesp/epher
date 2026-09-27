# The epher run command for Sublime Text (ADR-0069: the text-first
# editors run the same `epher/run` request the VS Code results pane
# uses). Everything else -- starting the server, hover, completion,
# diagnostics -- stays the LSP package's job; this module only adds
# the command, the results view, and the graph hand-off.
#
# The syntax and the client configuration come from the other files
# in this folder (epher.tmLanguage, LSP-epher.sublime-settings); see
# README.md for the install steps.

import os
import subprocess
import time

import sublime

from LSP.plugin import AbstractPlugin
from LSP.plugin import register_plugin
from LSP.plugin import unregister_plugin
from LSP.plugin.core.protocol import Request
from LSP.plugin.core.registry import LspTextCommand
from LSP.plugin.core.views import uri_from_view


class EpherPlugin(AbstractPlugin):
    @classmethod
    def name(cls) -> str:
        return "epher"


def plugin_loaded() -> None:
    register_plugin(EpherPlugin)


def plugin_unloaded() -> None:
    unregister_plugin(EpherPlugin)


# The view of the last run, so running again from the results view
# re-runs the script the results belong to (mirrors the emacs and
# nvim clients).
_last_script_view = None


def _results_dir() -> str:
    return os.path.join(sublime.cache_path(), "epher", "runs")


def _open_path(path: str) -> None:
    try:
        if sublime.platform() == "osx":
            subprocess.Popen(["open", path])
        elif sublime.platform() == "windows":
            os.startfile(path)  # type: ignore[attr-defined]  # noqa: S9
        else:
            subprocess.Popen(["xdg-open", path])
    except OSError as err:
        print("epher: could not open {}: {}".format(path, err))


class LspEpherRunCommand(LspTextCommand):
    """Runs the current epher script and shows the results pane.

    Every statement that produced something gets a row; error rows
    are marked red; graphs are written as SVG files under the
    editor's cache folder and opened with the system viewer.
    """

    session_name = "epher"

    def run(self, edit) -> None:
        global _last_script_view
        view = self.view
        if view.syntax() and view.syntax().scope != "source.epher":
            # Running from the results view (where a run leaves the
            # focus) re-runs the script the view belongs to.
            if _last_script_view is not None and _last_script_view.is_valid():
                view = _last_script_view
            else:
                sublime.status_message("epher: the current view is not an epher script")
                return
        _last_script_view = view
        session = self.session_by_name(self.session_name)
        if session is None:
            sublime.status_message("epher: the language server is not running")
            return
        params = {"textDocument": {"uri": uri_from_view(view)}}
        # Callbacks run on the worker thread; the view work below
        # hops to the UI thread like the LSP package's own commands.
        session.send_request(
            Request("epher/run", params, view),
            lambda result: sublime.set_timeout(lambda: self._on_result(result), 0),
            lambda error: sublime.set_timeout(lambda: self._on_error(error), 0),
        )

    def _on_error(self, error) -> None:
        message = error.get("message", "unknown error") if isinstance(error, dict) else str(error)
        sublime.error_message("epher run failed: {}".format(message))

    def _on_result(self, result) -> None:
        result = result or {}
        rows = []  # type: list
        src_lines = {}  # row (1-based) -> source line, 0 when no jump
        graphs = {}  # row (1-based) -> svg path
        error_rows = []
        for statement in result.get("statements") or []:
            display = statement.get("display")
            if display is None:
                continue
            # The wire carries 0-based lines; humans count from 1.
            rows.append("L{:<5} {}".format(statement.get("line", 0) + 1, display))
            src_lines[len(rows)] = statement.get("line", 0) + 1
            graphs[len(rows)] = ""
            if statement.get("error"):
                error_rows.append(len(rows))
        svgs = result.get("svgs") or []
        if svgs:
            directory = _results_dir()
            os.makedirs(directory, exist_ok=True)
            rows.append("")
            rows.append("Graphs ({}, each opened with the system viewer):".format(len(svgs)))
            for i, svg in enumerate(svgs, 1):
                path = os.path.join(directory, "run-{}-{}.svg".format(int(time.time()), i))
                try:
                    with open(path, "w", encoding="utf-8") as file:
                        file.write(svg)
                except OSError as err:
                    print("epher: could not write {}: {}".format(path, err))
                    continue
                rows.append("  " + path)
                graphs[len(rows)] = path
                _open_path(path)
        if not rows:
            rows = ["No output."]

        window = self.view.window() or sublime.active_window()
        results = window.new_file()
        results.set_scratch(True)
        results.set_name("epher run results")
        results.set_syntax_file("Packages/Text/Plain text.tmLanguage")
        results.run_command("append", {"characters": "\n".join(rows) + "\n"})
        results.set_read_only(True)
        for row in error_rows:
            line_start = results.text_point(row - 1, 0)
            line_region = results.full_line(line_start)
            results.add_regions(
                "epher-error-{}".format(row), [line_region], "region.redish", "", sublime.DRAW_NO_FILL
            )
        results.settings().set("epher_src_lines", src_lines)
        results.settings().set("epher_graphs", graphs)
