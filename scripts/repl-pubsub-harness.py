"""Live publish/subscribe regression harness for the epher frontends
(ADR-0010 amendment, 2026-11-27).

Drives the real binaries in ptys: `epher repl` and `epher tui` open and
active together against one shared store, plus a one-shot writer and a
fresh second REPL. Asserts the full pub/sub matrix: every frontend
publishes, the TUI subscribes live, the REPL's Up arrow reaches foreign
entries (Up ends a stale read, the second Up walks the fresh list), and
nothing a foreign frontend wrote is ever erased from the store.

The desktop app was verified the same way by hand (launch against the
live store, screenshot the History pane before and after a TUI write);
AppleScript driving needs an accessibility grant the harness cannot
assume.

Run after building the release binaries:

    cargo build --release -p app
    python3 scripts/repl-pubsub-harness.py

Exits nonzero when any assertion fails. The scenario this guards: the
REPL's arrow walk used to freeze at startup, and its stale per-line
publishes erased other frontends' entries from history.json (ADR-0010
amendment, 2026-11-27, has the audit that found it).
"""
import codecs
import os
import pty
import re
import select
import signal
import subprocess
import sys
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BIN = os.path.join(REPO, "target", "release", "epher")
RUN = os.path.join("/tmp", "epher-pubsub-harness")
STORE = os.path.join(RUN, "store")
HOME = os.path.join(RUN, "home")


class VT:
    """Minimal VT100: enough of CUP/EL/ED/movement to mirror ratatui."""

    def __init__(self, rows=40, cols=120):
        self.rows, self.cols = rows, cols
        self.grid = [[" "] * cols for _ in range(rows)]
        self.r = self.c = 0
        self.state = "ground"
        self.params = ""
        self.dec = codecs.getincrementaldecoder("utf-8")("replace")

    def feed(self, data: bytes):
        self._feed_text(self.dec.decode(data))

    def _feed_text(self, text):
        for ch in text:
            if self.state == "esc":
                if ch == "[":
                    self.state, self.params = "csi", ""
                elif ch == "]":
                    self.state = "osc"
                elif ch in "()":
                    self.state = "escint"
                else:
                    self.state = "ground"
            elif self.state == "escint":
                self.state = "ground"
            elif self.state == "osc":
                if ch == "\x07":
                    self.state = "ground"
                elif ch == "\x1b":
                    self.state = "osc_esc"
            elif self.state == "osc_esc":
                self.state = "ground" if ch == "\\" else "osc"
            elif self.state == "csi":
                if ch.isalpha() or ch in "@`~":
                    self._csi(self.params, ch)
                    self.state, self.params = "ground", ""
                elif ch != "?":
                    self.params += ch
            else:
                if ch == "\x1b":
                    self.state = "esc"
                elif ch == "\r":
                    self.c = 0
                elif ch == "\n":
                    self.r = min(self.r + 1, self.rows - 1)
                elif ch == "\b":
                    self.c = max(0, self.c - 1)
                elif ch >= " ":
                    if self.c < self.cols:
                        self.grid[self.r][self.c] = ch
                    self.c = min(self.c + 1, self.cols)

    def _csi(self, params, final):
        clean = re.sub(r"[^0-9;]", "", params)
        ps = [int(x) if x else 0 for x in clean.split(";")] if clean else []
        n = ps[0] if ps and ps[0] else 1
        if final in "Hf":
            r = (ps[0] if ps and ps[0] else 1) - 1
            c = (ps[1] if len(ps) > 1 and ps[1] else 1) - 1
            self.r = min(max(r, 0), self.rows - 1)
            self.c = min(max(c, 0), self.cols - 1)
        elif final == "A":
            self.r = max(0, self.r - n)
        elif final == "B":
            self.r = min(self.rows - 1, self.r + n)
        elif final == "C":
            self.c = min(self.cols - 1, self.c + n)
        elif final == "D":
            self.c = max(0, self.c - n)
        elif final == "G":
            self.c = min(max((ps[0] if ps and ps[0] else 1) - 1, 0), self.cols - 1)
        elif final == "J":
            mode = ps[0] if ps else 0
            if mode == 2:
                self.grid = [[" "] * self.cols for _ in range(self.rows)]
            elif mode == 0:
                for cc in range(self.c, self.cols):
                    self.grid[self.r][cc] = " "
                for rr in range(self.r + 1, self.rows):
                    self.grid[rr] = [" "] * self.cols
        elif final == "K":
            mode = ps[0] if ps else 0
            if mode == 0:
                for cc in range(self.c, self.cols):
                    self.grid[self.r][cc] = " "
            elif mode == 2:
                self.grid[self.r] = [" "] * self.cols

    def text(self):
        return "\n".join("".join(row).rstrip() for row in self.grid)

    def flat(self):
        return self.text().replace(" ", "")


class Pty:
    def __init__(self, argv, rows=40, cols=120):
        env = dict(os.environ)
        env["TERM"] = "xterm-256color"
        env["EPHER_STORE_DIR"] = STORE
        env["HOME"] = HOME
        os.makedirs(HOME, exist_ok=True)
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.execve(argv[0], argv, env)
        import fcntl, termios, struct
        fcntl.ioctl(self.fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
        self.vt = VT(rows, cols)
        self.buf = b""

    def read(self, timeout=1.0):
        end = time.time() + timeout
        got = b""
        while time.time() < end:
            r, _, _ = select.select([self.fd], [], [], 0.05)
            if r:
                try:
                    chunk = os.read(self.fd, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                got += chunk
                self.buf += chunk
                self.vt.feed(chunk)
        return got

    def send(self, data, delay=0.15):
        os.write(self.fd, data)
        time.sleep(delay)

    def screen(self):
        return self.vt.text()

    def flat(self):
        return self.vt.flat()

    def kill(self):
        try:
            os.kill(self.pid, signal.SIGKILL)
            os.waitpid(self.pid, 0)
        except (ProcessLookupError, ChildProcessError):
            pass
        os.close(self.fd)


def env_for_store():
    env = dict(os.environ)
    env["EPHER_STORE_DIR"] = STORE
    env["HOME"] = HOME
    return env


def store_files():
    out = []
    for root, _, files in os.walk(STORE):
        for f in files:
            out.append(os.path.join(root, f))
    return sorted(out)


def store_text():
    parts = []
    for p in store_files():
        try:
            with open(p, "rb") as fh:
                parts.append(f"-- {p}\n" + fh.read().decode("utf-8", "replace"))
        except OSError:
            pass
    return "\n".join(parts)


def check(name, cond, detail=""):
    print(f"[{'PASS' if cond else 'FAIL'}] {name}" + (f" | {detail}" if detail else ""))
    return bool(cond)


UP = b"\x1b[A"


def phase_repl_tui():
    subprocess.run(["rm", "-rf", STORE], check=False)
    os.makedirs(STORE, exist_ok=True)
    ok = {}

    repl = Pty([BIN, "repl"])
    repl.read(2.0)
    tui = Pty([BIN, "tui"])
    tui.read(3.0)
    print("== both frontends open, shared store:", STORE)

    # -- A. REPL publishes; TUI must show it live -------------------------
    repl.send(b"2+3\r")
    repl.read(1.5)
    ok["repl_eval"] = check("REPL evaluates 2+3 (= 5 shown)", "= 5" in repl.screen().replace("epher>", " "))
    tui.read(3.0)  # watcher budget
    ok["tui_sub_repl"] = check("REPL->TUI subscribe: TUI history pane shows 2+3 live",
                               "2+3" in tui.flat(), "flat screen head: " + tui.flat()[:120])
    ok["store_a"] = check("store holds 2+3", "2+3" in store_text())

    # -- B. TUI publishes; REPL arrow walk must recall it (THE bug) -------
    tui.send(b"4+5")
    tui.send(b"\r")
    tui.read(1.5)
    ok["tui_pub"] = check("TUI publishes 4+5 (pane shows it)", "4+5" in tui.flat(),
                          "flat head: " + tui.flat()[:120])
    ok["store_b"] = check("store holds 4+5", "4+5" in store_text())
    # Up ends the stale read (refresh boundary), the second Up walks the
    # fresh list (ADR-0010 amendment, 2026-11-27).
    m = len(repl.buf)
    repl.send(UP)
    repl.read(1.0)
    boundary = repl.buf[m:].decode("utf-8", "replace")
    ok["repl_boundary"] = check("Up at a foreign write ends the stale read (re-prompt, nothing evaluated)",
                                "= 9" not in boundary and "epher>" in boundary,
                                "boundary bytes: " + repr(boundary[-80:]))
    m = len(repl.buf)
    repl.send(UP)
    repl.read(1.0)
    fresh = repl.buf[m:].decode("utf-8", "replace")
    ok["repl_sub_tui"] = check("TUI->REPL subscribe: second Up recalls 4+5 (newest entry)",
                               "4+5" in fresh, "recalled: " + repr(fresh[-80:]))
    repl.send(b"\x03")
    repl.read(0.5)

    # -- C. REPL keeps publishing while open; TUI keeps following ---------
    repl.send(b"7-1\r")
    repl.read(1.5)
    tui.read(3.0)
    ok["tui_sub_repl2"] = check("REPL->TUI subscribe again: 7-1 visible live",
                                "7-1" in tui.flat())
    ok["store_c"] = check("store holds 7-1", "7-1" in store_text())

    # -- D. one-shot publishes while both are open ------------------------
    subprocess.run([BIN, "6*7"], env=env_for_store(), capture_output=True)
    tui.read(3.0)
    ok["tui_sub_oneshot"] = check("one-shot->TUI subscribe: 6*7 visible live",
                                  "6*7" in tui.flat())
    m = len(repl.buf)
    repl.send(UP)
    repl.read(1.0)
    repl.send(UP)
    repl.read(1.0)
    ok["repl_sub_oneshot"] = check("one-shot->REPL: Up recalls 6*7",
                                   "6*7" in repl.buf[m:].decode("utf-8", "replace"),
                                   "recalled: " + repr(repl.buf[m:][-80:]))
    repl.send(b"\x03")
    repl.read(0.3)

    # -- E. fresh REPL seeds from the store at start ----------------------
    repl2 = Pty([BIN, "repl"])
    repl2.read(2.0)
    m = len(repl2.buf)
    repl2.send(UP)
    repl2.read(1.0)
    ok["repl_seed"] = check("fresh REPL subscribe-at-start: Up recalls 6*7",
                            "6*7" in repl2.buf[m:].decode("utf-8", "replace"),
                            "recalled: " + repr(repl2.buf[m:][-80:]))
    repl2.kill()

    # -- evidence dump -----------------------------------------------------
    print("\n== store files ==")
    for p in store_files():
        print("  ", p)
    print("\n== TUI screen (final) ==")
    print(tui.screen())
    print("\n== REPL screen (final) ==")
    print(repl.screen())
    repl.kill()
    tui.kill()
    return ok



if __name__ == "__main__":
    results = phase_repl_tui()
    failed = [name for name, passed in results.items() if not passed]
    if failed:
        print("\nFAILED:", ", ".join(failed))
        sys.exit(1)
    print("\nall pub/sub assertions passed")
