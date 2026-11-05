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
import sublime_plugin

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
_SERVER_VERSION = "0.9.5"

# The results view's wrap (ADR-0078 R2): rows pre-broken at the
# wire's soft-break segments to this width, continuation rows indented
# the "L<line> " label's width, so a value never breaks mid-number
# however narrow the pane. The view's own word_wrap stays untouched.
_RESULTS_WIDTH = 100
_RESULTS_HANG = 7

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
        failure = None
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
            os.startfile(path)  # type: ignore[attr-defined]
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
        startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW  # type: ignore[attr-defined]
        startupinfo.wShowWindow = subprocess.SW_HIDE  # type: ignore[attr-defined]
    subprocess.Popen(args, startupinfo=startupinfo)


def _grid_body(grid: dict, hang: int) -> list:
    """One matrix as padded grid rows (ADR-0078 R3): cells padded
    to their column's width, columns joined with two spaces, the
    wire prefix on the first row, the suffix on the last,
    continuation rows indented hang spaces."""
    cells = grid.get("cells") or []
    widths = grid.get("widths") or []
    prefix = grid.get("prefix", "")
    suffix = grid.get("suffix", "")
    out = []
    last = len(cells)
    for i, row in enumerate(cells, 1):
        padded = []
        for c, cell in enumerate(row):
            w = widths[c] if c < len(widths) else len(cell)
            padded.append(" " * max(0, w - len(cell)) + cell)
        line = (prefix if i == 1 else " " * hang) + "  ".join(padded)
        if i == last:
            line += suffix
        out.append(line)
    if not out:
        out = [prefix + suffix]
    return out


def _list_body(list_data: dict, width: int, hang: int) -> tuple:
    """A long list folded at the threshold (ADR-0078 R3): shown
    elements with a (+n more) note, the hidden tail on its own rows
    wrapped on element boundaries to width. Returns (rows,
    tail_rows), the tail rows sitting at the rows' end."""
    elements = list_data.get("elements") or []
    hidden = list_data.get("hidden", 0) or 0
    shown_count = max(0, len(elements) - hidden)
    shown = elements[:shown_count]
    note = " (+{} more)".format(hidden) if hidden > 0 else ""
    head = "{}{{ {} }}{}{}".format(
        list_data.get("prefix", ""), ", ".join(shown), note,
        list_data.get("suffix", ""))
    rows = [head]
    tail_rows = 0
    if hidden > 0:
        line = None
        for i in range(shown_count, len(elements)):
            piece = elements[i]
            if line is None:
                line = piece
            elif len(line) + 2 + len(piece) <= width - hang:
                line = line + ", " + piece
            else:
                rows.append(" " * hang + line)
                tail_rows += 1
                line = piece
        if line is not None:
            rows.append(" " * hang + line)
            tail_rows += 1
    return rows, tail_rows


def _wrap_segments(segments, width: int, hang: int, plain_text: str) -> list:
    """The terminal pane's wrap (ADR-0078 R2) over the wire's
    soft-break segments: rows no wider than width, continuation
    rows indented hang spaces, breaks never inside a number.
    plain_text is the pane's display text, whose `= ` voice the
    first segment may need to shed."""
    width = max(width, 8)
    indent = " " * min(hang, width)
    segs = list(segments or [])
    if segs and plain_text and not plain_text.startswith("= ") and segs[0].startswith("= "):
        segs[0] = segs[0][2:]
    rows = []
    current = ""
    has_content = False
    continuation = False
    for seg in segs:
        for word in seg.split(" "):
            if not word:
                continue
            rest = word
            while rest:
                if not has_content and continuation and not current:
                    current = indent
                fresh = width - len(indent) if (not has_content and continuation) else width
                budget = width - len(current) - 1 if has_content else fresh
                if budget < 1:
                    budget = 1
                length = len(rest)
                if length <= budget:
                    take = length
                elif length <= fresh:
                    take = 0
                else:
                    take = budget
                if take <= 0:
                    if has_content:
                        rows.append(current)
                        current = ""
                        has_content = False
                    continuation = True
                else:
                    tail = rest[take:] if length > take else ""
                    if has_content:
                        current += " "
                    current += rest[:take]
                    has_content = True
                    rest = tail
                    if rest:
                        rows.append(current)
                        current = ""
                        has_content = False
                        continuation = True
    if has_content:
        rows.append(current)
    if not rows:
        rows = [""]
    return rows


def _pane_rows(pane: dict, width: int, hang: int) -> tuple:
    """One wire presentation line's display rows and how many rows
    hide behind the fold (the hidden list tail)."""
    if pane.get("grid"):
        return _grid_body(pane["grid"], hang), 0
    if pane.get("list"):
        return _list_body(pane["list"], width, hang)
    return _wrap_segments(pane.get("segments"), width, hang, pane.get("text", "")), 0


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
        # One record per display row: the shaped text, its tier, the
        # verbatim output it came from, whether it carries the run's
        # answer, and how many rows hide behind a fold that starts on
        # it (the hidden list tail, ADR-0078 R3). Without a
        # presentation (an older server) the verbatim display stands
        # alone, as before.
        records = []  # type: list
        graphs = {}  # result row (1-based) -> svg path
        for statement in result.get("statements") or []:
            display = statement.get("display")
            if display is None:
                continue
            # The wire carries 0-based lines; humans count from 1.
            label = "L{:<5} ".format(statement.get("line", 0) + 1)
            src_line = statement.get("line", 0) + 1
            presentation = statement.get("presentation")
            answer = bool(presentation and presentation.get("answer"))
            lines = presentation.get("lines") if presentation else None
            if lines:
                for pane in lines:
                    glyph = ""
                    verdict = pane.get("verdict")
                    if verdict is True:
                        glyph = "✓ "
                    elif verdict is False:
                        glyph = "✗ "
                    body, fold = _pane_rows(pane, _RESULTS_WIDTH, _RESULTS_HANG)
                    tail_start = len(body) - fold
                    for i, line in enumerate(body):
                        records.append({
                            "text": (label + glyph + line) if i == 0 else line,
                            "kind": pane.get("kind", "labeled"),
                            "verbatim": display,
                            "src_line": src_line,
                            "answer": answer,
                            "fold": fold if (i == tail_start and fold > 0) else 0,
                        })
            else:
                records.append({
                    "text": label + display,
                    "kind": "error" if statement.get("error") else "labeled",
                    "verbatim": display,
                    "src_line": src_line,
                    "answer": False,
                    "fold": 0,
                })
        svgs = result.get("svgs") or []
        if svgs:
            directory = _results_dir()
            os.makedirs(directory, exist_ok=True)
            records.append({
                "text": "", "kind": None, "verbatim": "",
                "src_line": 0, "answer": False, "fold": 0,
            })
            records.append({
                "text": "Graphs ({}, each opened with the system viewer):".format(len(svgs)),
                "kind": None, "verbatim": "", "src_line": 0,
                "answer": False, "fold": 0,
            })
            for i, svg in enumerate(svgs, 1):
                path = os.path.join(directory, "run-{}-{}.svg".format(int(time.time()), i))
                try:
                    with open(path, "w", encoding="utf-8") as file:
                        file.write(svg)
                except OSError as err:
                    print("LSP-epher: could not write {}: {}".format(path, err))
                    continue
                records.append({
                    "text": "  " + path, "kind": None, "verbatim": "",
                    "src_line": 0, "answer": False, "fold": 0,
                })
                graphs[len(records)] = path
                _open_path(path)
        if not records:
            records.append({
                "text": "No output.", "kind": None, "verbatim": "",
                "src_line": 0, "answer": False, "fold": 0,
            })

        window = self.view.window() or sublime.active_window()
        results = window.new_file()
        results.set_scratch(True)
        results.set_name("epher run results")
        results.set_syntax_file("Packages/Text/Plain text.tmLanguage")
        results.run_command("append", {
            "characters": "\n".join(record["text"] for record in records) + "\n"})
        results.set_read_only(True)

        def line_region(row: int) -> sublime.Region:
            return results.full_line(results.text_point(row - 1, 0))

        # The tiers (ADR-0078) and the answer emphasis (R4), as the
        # color scheme's own scopes: setup dims to the comment color,
        # a failure stays red (ADR-0079), the answer pops green.
        setup_rows = [r for r, rec in enumerate(records, 1) if rec["kind"] == "setup"]
        error_rows = [r for r, rec in enumerate(records, 1) if rec["kind"] == "error"]
        answer_rows = [r for r, rec in enumerate(records, 1) if rec["answer"]]
        flags = sublime.DRAW_NO_FILL | sublime.DRAW_NO_OUTLINE
        if setup_rows:
            results.add_regions("epher-setup", [line_region(r) for r in setup_rows],
                                "comment", "", flags)
        if error_rows:
            results.add_regions("epher-error", [line_region(r) for r in error_rows],
                                "region.redish", "", flags)
        if answer_rows:
            results.add_regions("epher-answer", [line_region(r) for r in answer_rows],
                                "region.greenish", "", flags)

        # The hidden list tails fold away (ADR-0078 R3), the view's
        # own disclosure.
        for r, rec in enumerate(records, 1):
            if rec["fold"]:
                beg = results.text_point(r - 1, 0)
                end = results.text_point(r - 1 + rec["fold"], 0)
                try:
                    results.fold(sublime.Region(beg, end))
                except Exception as err:
                    print("LSP-epher: could not fold the hidden tail: {}".format(err))

        # The run's answer carries the weight (ADR-0078 R4/R5): the
        # cursor lands on it, so the view opens at what matters.
        if answer_rows:
            point = results.text_point(answer_rows[0] - 1, 0)
            results.sel().clear()
            results.sel().add(sublime.Region(point, point))
            results.show(point)

        # The verbatim output rides every row for copy, and on hover
        # where the shaped text hides it (wrapped, gridded, folded).
        # Region annotations are a Sublime 4107+ parameter; older
        # builds skip the hover, the copy keeps working.
        results.settings().set("epher_src_view_id", view.id())
        results.settings().set("epher_src_lines",
                              {r: rec["src_line"] for r, rec in enumerate(records, 1)})
        results.settings().set("epher_graphs", graphs)
        results.settings().set("epher_verbatim",
                              {str(r): rec["verbatim"] for r, rec in enumerate(records, 1)
                               if rec["verbatim"]})
        hover_rows = [r for r, rec in enumerate(records, 1)
                      if rec["verbatim"] and rec["verbatim"] != rec["text"]]
        if hover_rows:
            try:
                results.add_regions(
                    "epher-verbatim", [line_region(r) for r in hover_rows],
                    "comment", "", flags,
                    [records[r - 1]["verbatim"] for r in hover_rows], "")
            except TypeError:
                pass


class EpherCopyVerbatimCommand(sublime_plugin.TextCommand):
    """Copies the row at the cursor's verbatim statement output
    (ADR-0078's first rule: copy never sees the shaped text)."""

    def run(self, edit) -> None:
        verbatim = self.view.settings().get("epher_verbatim") or {}
        for sel in self.view.sel():
            row = self.view.rowcol(sel.begin())[0] + 1
            value = verbatim.get(str(row)) or verbatim.get(row)
            if value:
                sublime.set_clipboard(value)
                sublime.status_message("LSP-epher: copied the verbatim output")
                return
        sublime.status_message("LSP-epher: nothing verbatim on this row")
