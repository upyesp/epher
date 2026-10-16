#!/usr/bin/env python3
"""Harness: drive the real epher-lsp with the EXACT byte sequence
EpherOneShot.kt speaks (initialize -> initialized -> didOpen ->
epher/run -> shutdown -> exit), for every candidate URI shape the
JetBrains plugin can produce on Windows. Any shape that makes the
process exit before the epher/run answer is the field bug.

Usage: python3 uri_harness.py /path/to/epher-lsp [--all]
"""
import json
import subprocess
import sys
import threading
import time

TEXT = '2 + 2\n"hello, " + "world"\n'


def frame(body: bytes) -> bytes:
    return b"Content-Length: %d\r\n\r\n%s" % (len(body), body)


def send(proc, payload):
    proc.stdin.write(frame(json.dumps(payload).encode()))
    proc.stdin.flush()


class Reader(threading.Thread):
    """Collects framed stdout messages; records EOF."""

    def __init__(self, proc):
        super().__init__(daemon=True)
        self.proc = proc
        self.messages = []
        self.eof = threading.Event()
        self.read_failure = None

    def run(self):
        input = self.proc.stdout
        while True:
            length = -1
            while True:
                line = input.readline()
                if line == b"":
                    # EOF mid-stream: exactly EpherOneShot's readFailure=EOF.
                    self.eof.set()
                    return
                if line in (b"\r\n", b"\n"):
                    break
                if line.lower().startswith(b"content-length:"):
                    length = int(line.split(b":")[1].strip())
            body = input.read(length)
            if len(body) < length:
                self.eof.set()
                return
            try:
                self.messages.append(json.loads(body))
            except json.JSONDecodeError as e:
                self.read_failure = str(e)
                self.eof.set()
                return


def wait_for(reader, rid, timeout):
    """The response with the given id, or None at EOF/timeout."""
    deadline = time.monotonic() + timeout
    while True:
        for m in reader.messages:
            if m.get("id") == rid:
                return m
        if reader.eof.is_set() or time.monotonic() > deadline:
            return None
        time.sleep(0.02)


def converse(binary, root_uri, doc_uri, timeout=10.0):
    proc = subprocess.Popen(
        [binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    reader = Reader(proc)
    reader.start()

    # initialize — exactly EpherOneShot's shape (processId null, rootUri,
    # empty capabilities), and waited for like it does.
    send(proc, {"jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"processId": None, "rootUri": root_uri,
                           "capabilities": {}}})
    wait_for(reader, 1, timeout)
    # initialized notification
    send(proc, {"jsonrpc": "2.0", "method": "initialized", "params": {}})
    # didOpen with the document uri under test
    send(proc, {"jsonrpc": "2.0", "method": "textDocument/didOpen",
                "params": {"textDocument": {"uri": doc_uri,
                                            "languageId": "epher",
                                            "version": 1, "text": TEXT}}})
    # epher/run
    send(proc, {"jsonrpc": "2.0", "id": 2, "method": "epher/run",
                "params": {"textDocument": {"uri": doc_uri}}})

    got = wait_for(reader, 2, timeout)

    # polite shutdown, then exit (broken pipe tolerated, like EpherOneShot)
    try:
        send(proc, {"jsonrpc": "2.0", "id": 3, "method": "shutdown",
                    "params": None})
        send(proc, {"jsonrpc": "2.0", "method": "exit", "params": {}})
    except (BrokenPipeError, OSError):
        pass
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()

    stderr = proc.stderr.read().decode(errors="replace")
    return got, proc.returncode, stderr


SHAPES = [
    # IntelliJ VirtualFile.getUrl() Windows shape: drive in the
    # authority position, two slashes.
    ("file://C:/Users/pete/proj", "file://C:/Users/pete/proj/eclipse.epher"),
    # Canonical LSP shape: empty authority, three slashes.
    ("file:///C:/Users/pete/proj", "file:///C:/Users/pete/proj/eclipse.epher"),
    # java.io.File.toURI() shape: single slash after scheme.
    ("file:/C:/Users/pete/proj", "file:/C:/Users/pete/proj/eclipse.epher"),
    # Lowercase drive.
    ("file:///c:/users/pete/proj", "file:///c:/users/pete/proj/eclipse.epher"),
    # The shape the platform's own LSP client and VS Code send on
    # Windows: lowercased drive, escaped colon.
    ("file:///c%3A/users/pete/proj", "file:///c%3A/users/pete/proj/eclipse.epher"),
    # Document outside the project root.
    ("file:///C:/Users/pete", "file:///C:/Users/pete/tmp/eclipse.epher"),
    # Backslashes in the URI.
    ("file:///C:\\Users\\pete\\proj", "file:///C:\\Users\\pete\\proj\\eclipse.epher"),
    # Raw space in the path (directory with a space in its name).
    ("file:///C:/Users/pete my/proj", "file:///C:/Users/pete my/proj/eclipse.epher"),
    # Percent-encoded space — the canonical shape java.nio's toUri (the
    # fixed EpherUris) emits for that same directory.
    ("file:///C:/Users/pete%20my/proj", "file:///C:/Users/pete%20my/proj/eclipse.epher"),
    # Raw non-ASCII in the path.
    ("file:///C:/Users/José/proj", "file:///C:/Users/José/proj/eclipse.epher"),
    # Percent-encoded non-ASCII — the canonical shape for that same name.
    ("file:///C:/Users/Jos%C3%A9/proj", "file:///C:/Users/Jos%C3%A9/proj/eclipse.epher"),
]


def main():
    binary = sys.argv[1]
    only_dying = "--all" not in sys.argv
    died = []
    for root_uri, doc_uri in SHAPES:
        answer, code, stderr = converse(binary, root_uri, doc_uri)
        if answer is None:
            died.append((root_uri, doc_uri, code, stderr))
            status = "DIED  (exit %s)" % code
        elif answer.get("error"):
            status = "ERROR (rpc %s)" % answer["error"]["message"]
        else:
            lines = answer["result"]["statements"]
            status = "OK    (%d statements, svgs=%d)" % (
                len(lines), len(answer["result"]["svgs"]))
        print("%-48s %-22s %s" % (root_uri, doc_uri.rsplit("/", 1)[-1] or doc_uri, status))
        if "DIED" in status and stderr.strip():
            print("      stderr: %s" % " | ".join(stderr.strip().splitlines()))
    print()
    if died:
        print("SHAPES THAT KILL THE SERVER BEFORE epher/run:")
        for root_uri, doc_uri, code, stderr in died:
            print("  %s" % doc_uri)
        return 1
    print("no shape killed the server")
    return 0


if __name__ == "__main__":
    sys.exit(main())
