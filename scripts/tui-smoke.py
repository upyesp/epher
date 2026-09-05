#!/usr/bin/env python3
"""Drive the epher TUI through a real pty and assert what the screen shows.

Black-box smoke test for the full-screen terminal UI — the one frontend
CI cannot click. The pty plays the user; pyte (a terminal emulator)
plays the screen: every byte the TUI writes is replayed into a
120x40 cell grid, and assertions read the final cell state rather than
raw escape sequences, which ratatui rewrites in place.

Used by the macOS Apple silicon workflow (.github/workflows/macos-silicon.yml)
and handy on any unix:

    python3 scripts/tui-smoke.py target/release/epher

Exits 0 when every check passes; on failure prints the screen and
exits 1.
"""

import fcntl
import os
import pty
import select
import struct
import sys
import termios
import time

import pyte

COLS, ROWS = 120, 40


def fail(screen, message):
    print(f"TUI SMOKE FAIL: {message}")
    print("---- screen ----")
    for line in screen.display:
        print(line.rstrip())
    sys.exit(1)


def main():
    if len(sys.argv) < 2:
        print("usage: tui-smoke.py <epher-binary>")
        sys.exit(2)
    binary = os.path.abspath(sys.argv[1])

    screen = pyte.Screen(COLS, ROWS)
    stream = pyte.ByteStream(screen)

    pid, fd = pty.fork()
    if pid == 0:
        # Child: the slave pty is now stdin/stdout/stderr. Size it
        # before exec so ratatui's first frame sees the real grid.
        fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
        os.environ["TERM"] = "xterm-256color"
        os.environ["LANG"] = "en_US.UTF-8"
        os.environ.setdefault("EPHER_STORE_DIR", "/tmp/epher-tui-smoke-store")
        os.execv(binary, [binary, "tui"])

    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))

    def drain(seconds=0.2):
        """Feed the emulator for a while. False once the child is gone."""
        end = time.time() + seconds
        while time.time() < end:
            ready, _, _ = select.select([fd], [], [], 0.05)
            if ready:
                try:
                    data = os.read(fd, 65536)
                except OSError:
                    return False
                if not data:
                    return False
                stream.feed(data)
        return True

    def wait_for(text, timeout=15.0):
        end = time.time() + timeout
        while time.time() < end:
            if text in "\n".join(screen.display):
                return
            drain(0.1)
        fail(screen, f"timed out waiting for {text!r}")

    def send(data):
        os.write(fd, data)

    try:
        # Boot: the footer hint line names the quit key.
        drain(2.0)
        wait_for("Ctrl+C quit", 30)

        # A plain evaluation, typed and submitted.
        send(b'print("x:", 6*7)\r')
        wait_for("x: 42")

        # Powers.
        send(b"2 ^ 10\r")
        wait_for("1024")

        # A pasted script (the website flow, ADR-0059): the clipboard
        # arrives bracketed, block comment and blank lines included,
        # and one Enter runs the whole entry.
        paste = (
            b"\x1b[200~"
            b"/* === pasted banner ===\n"
            b"   a block comment spanning lines\n"
            b"   === */\n"
            b"\n"
            b'print("p1:", 5*2)\n'
            b'print("p2:", 2*3)'
            b"\x1b[201~"
        )
        send(paste)
        drain(1.0)
        send(b"\r")
        wait_for("p1: 10")
        wait_for("p2: 6")

        # History keeps the submitted lines.
        wait_for('print("x:", 6*7)')

        # Quit cleanly with Ctrl+C; the app catches it and exits 0.
        send(b"\x03")
        deadline = time.time() + 10
        status = None
        while time.time() < deadline:
            done, detail = os.waitpid(pid, os.WNOHANG)
            if done:
                status = detail
                break
            drain(0.1)
        if status is None:
            fail(screen, "the TUI did not exit on Ctrl+C")
        if not (os.WIFEXITED(status) and os.WEXITSTATUS(status) == 0):
            fail(screen, f"the TUI exited abnormally: status {status}")
    finally:
        try:
            os.close(fd)
        except OSError:
            pass

    print("TUI SMOKE OK")
    sys.exit(0)


if __name__ == "__main__":
    main()
