#!/usr/bin/env python3
"""Drive epher-tui through a pty and print its de-ANSI-fied render.

Called by scripts/a11y/native-terminals.mjs (ADR-0009 amendment
2026-11-03: the native-surface accessibility leg). The pty gets a real
window size and TERM, staged keystrokes drive one evaluation, one
graph, then quit; everything the TUI painted is printed as plain text
for assertion by the caller.
"""
import fcntl
import os
import pty
import re
import select
import struct
import sys
import termios
import time

BIN = sys.argv[1] if len(sys.argv) > 1 else "./target/debug/epher-tui"

pid, fd = pty.fork()
if pid == 0:
    os.environ["TERM"] = "xterm-256color"
    os.execv(BIN, [BIN])
    os._exit(1)
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 100, 0, 0))

buf = b""


def pump(seconds):
    global buf
    end = time.time() + seconds
    while time.time() < end:
        ready, _, _ = select.select([fd], [], [], 0.05)
        if ready:
            try:
                chunk = os.read(fd, 65536)
            except OSError:
                return False
            if not chunk:
                return False
            buf += chunk
    return True


pump(1.5)
os.write(fd, b"2+3*4\r")
pump(1.2)
os.write(fd, b"graph sin(x)\r")
pump(1.5)
os.write(fd, b"q")
pump(0.8)
try:
    os.kill(pid, 9)
except ProcessLookupError:
    pass
try:
    os.close(fd)
except OSError:
    pass

text = buf.decode("utf-8", "replace")
text = re.sub(r"\x1b\[[0-9;?]*[a-zA-Z]", "", text)
text = text.replace("\x00", "")
print(text)
