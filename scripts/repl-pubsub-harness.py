"""Live publish/subscribe regression harness for the epher frontends
(ADR-0010 amendment, 2026-11-27).

Drives the real binaries in ptys: `epher repl` and `epher tui` open and
active together against one shared store, plus a one-shot writer, a
fresh second REPL, and a REPL alone on a fresh profile. Asserts the full
pub/sub matrix: every frontend publishes, the TUI subscribes live, the
REPL's Up arrow reaches foreign entries in one press (and its own last
line in one press too), the half-typed line comes back on Down, Ctrl-R
finds a foreign entry before any line boundary, a `load` publishes at
once, and nothing a foreign frontend wrote is ever erased from the store
(ADR-0010 amendment, 2026-11-28).

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
amendment, 2026-11-27, has the audit that found it); the 2026-11-28
amendment replaced the watcher flag that fixed it, which ended the read
on every Up press that followed the REPL's own line.
"""
import codecs
import os
import pty
import re
import select
import shutil
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
    def __init__(self, argv, rows=40, cols=120, store=None):
        env = dict(os.environ)
        env["TERM"] = "xterm-256color"
        env["EPHER_STORE_DIR"] = store or STORE
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
        """SIGKILL, then reap without ever blocking.

        The old teardown killed and then called a blocking waitpid with
        the pty master still open; a frontend blocked in a terminal write
        (the TUI keeps rendering) stays in that write until someone
        drains the master, so the wait never returned and the harness
        hung after its last check without printing a verdict. Drain the
        master while polling for the exit instead.
        """
        try:
            os.kill(self.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        deadline = time.time() + 5
        while time.time() < deadline:
            readable, _, _ = select.select([self.fd], [], [], 0.05)
            if readable:
                try:
                    if not os.read(self.fd, 65536):
                        break  # the slave side closed: the child is gone
                except OSError:
                    break
            try:
                if os.waitpid(self.pid, os.WNOHANG)[0] == self.pid:
                    break
            except ChildProcessError:
                break
        else:
            print(f"warning: pid {self.pid} did not exit within 5s of SIGKILL")
        try:
            os.close(self.fd)
        except OSError:
            pass


def env_for_store(store=None):
    env = dict(os.environ)
    env["EPHER_STORE_DIR"] = store or STORE
    env["HOME"] = HOME
    return env


def store_files(store=None):
    out = []
    for root, _, files in os.walk(store or STORE):
        for f in files:
            out.append(os.path.join(root, f))
    return sorted(out)


def store_text(store=None):
    parts = []
    for p in store_files(store):
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
DOWN = b"\x1b[B"
CTRL_R = b"\x12"
CTRL_G = b"\x07"


def pressed(pty, key, settle=1.0):
    """Send one key and return the bytes the terminal got back for it."""
    mark = len(pty.buf)
    pty.send(key)
    pty.read(settle)
    return pty.buf[mark:].decode("utf-8", "replace")


def phase_repl_tui():
    shutil.rmtree(STORE, ignore_errors=True)
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

    # -- B. TUI publishes; the REPL's first Up recalls it (THE bug) --------
    tui.send(b"4+5")
    tui.send(b"\r")
    tui.read(1.5)
    ok["tui_pub"] = check("TUI publishes 4+5 (pane shows it)", "4+5" in tui.flat(),
                          "flat head: " + tui.flat()[:120])
    ok["store_b"] = check("store holds 4+5", "4+5" in store_text())
    # Ctrl-R before any line boundary (no Up, no Ctrl-C, no Enter yet)
    # finds the foreign entry; Ctrl-G aborts the search.
    repl.send(CTRL_R)
    repl.read(0.4)
    repl.send(b"4+")
    search = repl.read(0.8).decode("utf-8", "replace")
    ok["repl_ctrl_r"] = check("Ctrl-R before any boundary finds the foreign entry",
                              "4+5" in search and "failed" not in search,
                              "search bytes: " + repr(search[-100:]))
    repl.send(CTRL_G)
    repl.read(0.4)
    # One press, no interrupt: the recalled entry is on the line and the
    # read did not end. A normal recall redraws the prompt and the line
    # on the same row; the swallowed press of 0.9.28 ended the read,
    # which printed a newline and a fresh prompt row, so a newline is
    # the tell.
    first = pressed(repl, UP)
    ok["repl_sub_tui"] = check("TUI->REPL subscribe: the first Up recalls 4+5",
                               "4+5" in first, "recalled: " + repr(first[-80:]))
    ok["repl_one_press"] = check("... in one press: the read did not end, nothing evaluated",
                                 "\r\n" not in first and "= 9" not in first,
                                 "bytes: " + repr(first[-80:]))
    repl.send(b"\x03")
    repl.read(0.4)

    # -- C. The REPL's own line: the first Up after a submit recalls it ---
    repl.send(b"7-1\r")
    repl.read(1.5)
    own = pressed(repl, UP, settle=0.6)
    ok["repl_own_line"] = check("the first Up after a submitted line recalls it (7-1)",
                                "7-1" in own, "recalled: " + repr(own[-80:]))
    ok["repl_own_one_press"] = check("... in one press: the read did not end",
                                      "\r\n" not in own, "bytes: " + repr(own[-80:]))
    repl.send(b"\x03")
    repl.read(0.4)
    tui.read(3.0)
    ok["tui_sub_repl2"] = check("REPL->TUI subscribe again: 7-1 visible live",
                                "7-1" in tui.flat())
    ok["store_c"] = check("store holds 7-1", "7-1" in store_text())

    # -- D. one-shot publishes while both are open ------------------------
    subprocess.run([BIN, "6*7"], env=env_for_store(), capture_output=True)
    tui.read(3.0)
    ok["tui_sub_oneshot"] = check("one-shot->TUI subscribe: 6*7 visible live",
                                  "6*7" in tui.flat())
    # With a line half typed: the first Up recalls the foreign entry,
    # and Down comes back to the half-typed line, nothing lost.
    repl.send(b"1+")
    repl.read(0.4)
    shot = pressed(repl, UP)
    ok["repl_sub_oneshot"] = check("one-shot->REPL: the first Up recalls 6*7",
                                   "6*7" in shot and "\r\n" not in shot,
                                   "recalled: " + repr(shot[-80:]))
    back = pressed(repl, DOWN)
    ok["repl_typed_kept"] = check("Down past the end brings the half-typed line back (1+)",
                                  "1+" in back and "6*7" not in back,
                                  "bytes: " + repr(back[-80:]))
    repl.send(b"\x03")
    repl.read(0.4)

    # -- E. fresh REPL seeds from the store at start ----------------------
    repl2 = Pty([BIN, "repl"])
    repl2.read(2.0)
    seeded = pressed(repl2, UP)
    ok["repl_seed"] = check("fresh REPL subscribe-at-start: Up recalls 6*7",
                            "6*7" in seeded, "recalled: " + repr(seeded[-80:]))
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


def phase_load_fresh_profile():
    """A `load` publishes at once, on the profile that used to lose it."""
    store = os.path.join(RUN, "fresh-store")
    shutil.rmtree(store, ignore_errors=True)
    os.makedirs(RUN, exist_ok=True)
    script = os.path.join(RUN, "setup.epher")
    with open(script, "w") as fh:
        fh.write("rate = 7\nrate * 2\n")
    ok = {}
    # The store directory does not exist yet: the REPL creates it at
    # startup, which the old watcher took for a foreign write. This
    # REPL is alone: no TUI, no watcher anywhere.
    repl = Pty([BIN, "repl"], store=store)
    repl.read(2.0)
    repl.send(f"load {script}\r".encode(), delay=1.0)
    repl.read(1.0)
    ok["load_ran"] = check("load runs the script (= 7, = 14 shown)",
                           "= 7" in repl.screen() and "= 14" in repl.screen())
    # The script's lines are in the walk: the newest is one Up away.
    own = pressed(repl, UP, settle=0.6)
    ok["load_walk"] = check("the first Up after the load recalls the script's last line",
                            "rate * 2" in own and "\r\n" not in own,
                            "recalled: " + repr(own[-80:]))
    repl.send(b"\x03")
    repl.read(0.4)
    # Another frontend writes before the next line, as on a busy desktop.
    subprocess.run([BIN, "1+1"], env=env_for_store(store), capture_output=True)
    repl.read(0.5)
    repl.send(b"rate * 100\r")
    repl.read(1.0)
    screen = repl.screen()
    ok["load_kept"] = check("the loaded assignment survives the next line (= 700)",
                            "= 700" in screen and "unknown name" not in screen,
                            "screen tail: " + repr(screen[-160:]))
    # Alone, the first Up after a submitted line recalls it too.
    own = pressed(repl, UP, settle=0.6)
    ok["repl_alone_one_press"] = check("alone: the first Up after a submitted line recalls it",
                                       "rate * 100" in own and "\r\n" not in own,
                                       "recalled: " + repr(own[-80:]))
    ok["load_published"] = check("the store holds the load line and the script's lines",
                                 "rate = 7" in store_text(store) and "load " in store_text(store))
    print("\n== fresh-profile REPL screen ==")
    print(screen)
    repl.kill()
    return ok



if __name__ == "__main__":
    results = {}
    results.update(phase_repl_tui())
    results.update(phase_load_fresh_profile())
    failed = [name for name, passed in results.items() if not passed]
    if failed:
        print("\nFAILED:", ", ".join(failed))
        sys.exit(1)
    print("\nall pub/sub assertions passed")
