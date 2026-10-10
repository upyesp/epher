# The epher integration for Sublime Text: the LspPlugin registration,
# the managed server download, the run command, the results view, and
# the graph hand-off.
#
# Everything else -- starting the server, hover, completion,
# diagnostics, inlay hints -- is the LSP package's job, driven by
# LSP-epher.sublime-settings next to this file. The run command sends
# the same `epher/run` request the VS Code results pane uses (ADR-0069).
#
# The session name comes from the package name (`LSP-epher`), which is
# also where LSP looks for the settings file, so neither is configured
# here (LspPlugin API, LSP 2.11+).
#
# The server binary manages itself: on first start the plugin fetches
# the epher-lsp asset for this platform from the matching epher release
# into the package storage (LspPlugin.plugin_storage_path) and points
# the client command at it. A browser download would leave the file
# quarantined on macOS, which is exactly what this avoids; a `command`
# set in the settings file is always honored as-is and skips the
# download.
#
# The syntax comes from epher.tmLanguage in this folder; see README.md
# for the install steps, including the LSP package itself, which
# Package Control does not install automatically.

from __future__ import annotations

import gzip
import io
import os
import shutil
import subprocess
import time
import urllib.request
import zipfile
from pathlib import Path

import sublime

from LSP.plugin import LspPlugin
from LSP.plugin import LspTextCommand
from LSP.plugin import OnPreStartContext
from LSP.plugin import PluginStartError
from LSP.plugin import Request
from LSP.plugin import uri_from_view

# The epher release the managed download fetches, kept in lockstep with
# the client (ADR-0066: the extension version keys the server URL).
# listing/assemble-repo.sh rewrites this line from SUBLIME_SERVER_VERSION
# when the package repository is assembled for a v-tag; the value here
# is the current train, so a symlinked test folder resolves to today's
# release, with the latest-release fallback covering assemblies that
# run ahead of their tag.
_SERVER_VERSION = "0.5.77"

# The client command this package ships. Anything else found in the
# resolved configuration at start time means the user manages the
# binary themselves, and the download is skipped.
_DEFAULT_COMMAND = ["epher-lsp"]


class EpherPlugin(LspPlugin):
    """The epher language server, downloaded into the package storage.

    The name, the settings file, and the session come from the package
    name; this override only puts the managed binary in front of the
    start. LSP runs it on the worker thread, where blocking I/O such
    as a download is expected.
    """

    @classmethod
    def on_pre_start_async(cls, context: OnPreStartContext) -> None:
        if context.configuration.command != _DEFAULT_COMMAND:
            return
        managed = _managed_server()
        if managed is None:
            managed, failure = _download_server()
        if managed is not None:
            context.configuration.command = [str(managed)]
        elif shutil.which("epher-lsp") is None:
            # Nothing can start: say so through LSP's own start failure,
            # which carries the message, instead of a bare "no such file".
            raise PluginStartError(
                "the epher-lsp server could not be downloaded ({}) and no "
                "epher-lsp was found on PATH; check the network and reload "
                "the window, or install the binary manually (the README "
                "has the steps)".format(failure))


def plugin_loaded() -> None:
    EpherPlugin.register()


def plugin_unloaded() -> None:
    EpherPlugin.unregister()


def _binary_name() -> str:
    if sublime.platform() == "windows":
        return "epher-lsp.exe"
    return "epher-lsp"


# Sublime's arch names differ from the release asset names: the
# banner's "linux x64" is our linux-x86_64, and its "arm64" is aarch64.
_ARCHES = {"x64": "x86_64", "arm64": "aarch64"}


def _asset_name() -> "str | None":
    """The release asset for this platform, or None where no build
    exists yet (Intel macOS and Windows ARM64 join later, ADR-0066)."""
    arch = _ARCHES.get(sublime.arch())
    if arch is None:
        return None
    platform = {"osx": "macos", "linux": "linux", "windows": "windows"}[sublime.platform()]
    extension = ".zip" if sublime.platform() == "windows" else ".gz"
    return "epher-lsp-{}-{}{}".format(platform, arch, extension)


def _release_asset_url(asset: str) -> str:
    return "https://github.com/upyesp/epher/releases/download/v{}/{}".format(
        _SERVER_VERSION, asset)


def _latest_asset_url(asset: str) -> str:
    return "https://github.com/upyesp/epher/releases/latest/download/{}".format(asset)


def _managed_server() -> Path | None:
    """The managed binary, when one for this exact client version exists."""
    directory = EpherPlugin.plugin_storage_path
    binary = directory / _binary_name()
    stamp = directory / "{}.version".format(binary.name)
    try:
        if binary.is_file() and stamp.read_text(encoding="utf-8").strip() == _SERVER_VERSION:
            return binary
    except OSError:
        pass
    return None


def _fetch(url: str) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": "LSP-epher"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def _download_server() -> "tuple[Path | None, str | None]":
    """Fetch, unpack, and stamp the server binary for this platform.

    Returns (None, reason) on failure; the caller then either falls
    back to the configured command or reports the reason, so a machine
    with epher-lsp on PATH keeps working offline.
    """
    directory = EpherPlugin.plugin_storage_path
    asset = _asset_name()
    if asset is None:
        return None, "no epher-lsp build exists for {} {}".format(
            sublime.platform(), sublime.arch())
    try:
        data = _fetch(_release_asset_url(asset))
    except OSError as err:
        # A missing versioned asset is normal for a package tested from
        # the monorepo ahead of its release: fall back to the latest.
        print("LSP-epher: no v{} {} asset ({}); trying the latest release".format(
            _SERVER_VERSION, asset, err))
        try:
            data = _fetch(_latest_asset_url(asset))
        except OSError as err:
            return None, str(err)
    try:
        directory.mkdir(parents=True, exist_ok=True)
        binary = directory / _binary_name()
        if asset.endswith(".zip"):
            with zipfile.ZipFile(io.BytesIO(data)) as archive:
                members = [name for name in archive.namelist() if not name.endswith("/")]
                binary.write_bytes(archive.read(members[0]))
        else:
            binary.write_bytes(gzip.decompress(data))
        if sublime.platform() != "windows":
            binary.chmod(0o755)
        (directory / "{}.version".format(binary.name)).write_text(
            _SERVER_VERSION, encoding="utf-8")
        print("LSP-epher: installed epher-lsp {} ({})".format(_SERVER_VERSION, asset))
        return binary, None
    except (OSError, EOFError, zipfile.BadZipFile, IndexError) as err:
        return None, str(err)


# The results view carries the id of the view it describes
# (`epher_src_view_id`), so running from it re-runs the script the
# results belong to (the emacs and nvim pattern); a closed source
# simply does not resolve.

def _is_epher_view(view: sublime.View) -> bool:
    syntax = view.syntax()
    return syntax is not None and syntax.scope == "source.epher"


def _view_by_id(view_id: int) -> sublime.View | None:
    for window in sublime.windows():
        for view in window.views():
            if view.id() == view_id:
                return view
    return None


def _source_view_of(view: sublime.View) -> sublime.View | None:
    view_id = view.settings().get("epher_src_view_id")
    if isinstance(view_id, int):
        return _view_by_id(view_id)
    return None


def _results_dir() -> Path:
    # The package storage, not a bespoke cache folder: the directory is
    # created on demand and consistent with the other LSP packages.
    return EpherPlugin.plugin_storage_path / "runs"


def _open_path(path: str) -> None:
    try:
        if sublime.platform() == "osx":
            _popen(["open", path])
        elif sublime.platform() == "windows":
            os.startfile(path)  # type: ignore[attr-defined]  # noqa: S9
        else:
            _popen(["xdg-open", path])
    except OSError as err:
        print("LSP-epher: could not open {}: {}".format(path, err))


def _popen(args: "list[str]") -> None:
    """Popen with the Windows hidden-window handling the ST package
    reviewer asks for: without STARTF_USESHOWWINDOW the spawned
    opener would flash a console. The flag only exists on Windows,
    so it is built under the platform check; the branches above are
    osx/linux, but the reviewer reads the package, not the runtime.
    """
    startupinfo = None
    if sublime.platform() == "windows":
        startupinfo = subprocess.STARTUPINFO()  # type: ignore[attr-defined]
        startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW  # type: ignore[attr-defined]  # noqa: SIM
        startupinfo.wShowWindow = subprocess.SW_HIDE  # type: ignore[attr-defined]
    subprocess.Popen(args, startupinfo=startupinfo)


class LspEpherRunCommand(LspTextCommand):
    """Runs the current epher script and shows the results pane.

    Every statement that produced something gets a row; error rows
    are marked red; graphs are written as SVG files under the
    package storage and opened with the system viewer. Running from
    a results view re-runs the script that view's stored source id
    points at.
    """

    def is_enabled(self, event=None, point=None) -> bool:
        # The command is also useful from the results view, which has
        # no session of its own: there, a resolvable source view takes
        # its place. The base class check would grey the palette entry
        # out in exactly that case.
        if self.session_by_name():
            return True
        return _is_epher_view(self.view) or _source_view_of(self.view) is not None

    def run(self, edit) -> None:
        view = self.view
        if not _is_epher_view(view):
            source = _source_view_of(view)
            if source is None:
                sublime.status_message("LSP-epher: the current view is not an epher script")
                return
            view = source
        session = self.session_by_name()
        if session is None:
            sublime.status_message("LSP-epher: the language server is not running")
            return
        params = {"textDocument": {"uri": uri_from_view(view)}}
        # Callbacks run on the worker thread; the view work below
        # hops to the UI thread like the LSP package's own commands.
        session.send_request(
            Request("epher/run", params, view),
            lambda result: sublime.set_timeout(lambda: self._on_result(result, view), 0),
            lambda error: sublime.set_timeout(lambda: self._on_error(error), 0),
        )

    def _on_error(self, error) -> None:
        message = error.get("message", "unknown error") if isinstance(error, dict) else str(error)
        sublime.error_message("LSP-epher run failed: {}".format(message))

    def _on_result(self, result, view) -> None:
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
                    print("LSP-epher: could not write {}: {}".format(path, err))
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
        results.settings().set("epher_src_view_id", view.id())
        results.settings().set("epher_src_lines", src_lines)
        results.settings().set("epher_graphs", graphs)
