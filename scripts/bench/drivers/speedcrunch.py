#!/usr/bin/env python3
"""SpeedCrunch native driver (stage 3 pilot).

Instrument (proven in the stage 2 spikes and declared in the pilot README):
multiline paste into an empty, focused input line. SpeedCrunch evaluates
every pasted line into its history. Paste is the tool's real bulk-ingest
path; synthetic typing is avoided entirely (the autocomplete popup swallows
Return). Each timed pass delivers the dataset in fixed 120-line chunks:
    clear history (untimed) -> verify input empty -> Cmd+V -> poll until the
    history accessibility value shows exactly the chunk's results -> read,
    compare (verification) or add the chunk span to the pass total (timed).
The history accessibility value caps at about 4.5 KB (~180 entries), which
is why chunks are 120 lines. Completion per chunk is exact, so a swallowed
paste is detected, the chunk is retried, and retries are reported.

Syntax rendering: SpeedCrunch has no log() function; ln() is natural and
log10() is base-10 (verified 2026-11-07). The w2 dataset file passed here is
the committed ln-rendered copy of the canonical dataset (function-name-only
change, values identical, checked by the referee gate).

Boundary (declared): per-chunk span = Cmd+V posted -> all of the chunk's
results present in the history accessibility value; chunk spans are summed
into the pass time. History clears, ref focus, and clipboard writes are
outside the span.

Sample policy: warm-up 3 untimed passes, then the plan's bucket rule on the
warm-up duration. A chunk whose count does not arrive within 10 s is
retried up to 3 times; the pass aborts and retries wholesale if any chunk
still fails.
"""
import argparse
import json
import re
import subprocess
import sys
import time

import Quartz
from ApplicationServices import (
    AXUIElementCreateApplication,
    AXUIElementCopyAttributeValue,
    AXUIElementSetAttributeValue,
    AXUIElementPerformAction,
    AXValueGetValue,
    kAXChildrenAttribute,
    kAXFocusedAttribute,
    kAXMainWindowAttribute,
    kAXMenuBarAttribute,
    kAXPositionAttribute,
    kAXRoleAttribute,
    kAXTitleAttribute,
    kAXValueAttribute,
    kAXWindowsAttribute,
    kAXValueCGPointType,
)
from AppKit import NSPasteboard, NSPasteboardTypeString

V_KEY = 9
CMD = Quartz.kCGEventFlagMaskCommand
CHUNK = 120
HISTORY_CAP_BYTES = 4000  # stay under the observed ~4.5 KB AX cap


def ax_copy(el, attr):
    err, val = AXUIElementCopyAttributeValue(el, attr, None)
    return val if err == 0 else None


def find_textareas(el, out=None):
    if out is None:
        out = []
    role = str(ax_copy(el, kAXRoleAttribute) or "")
    if role == "AXTextArea":
        val = ax_copy(el, kAXValueAttribute)
        err, posv = AXUIElementCopyAttributeValue(el, kAXPositionAttribute, None)
        y = -1.0
        if err == 0 and posv is not None:
            ok, p = AXValueGetValue(posv, kAXValueCGPointType, None)
            if ok and p is not None:
                y = p.y
        out.append((el, len(str(val)) if val else 0, y))
    kids = ax_copy(el, kAXChildrenAttribute) or []
    for k in kids:
        find_textareas(k, out)
    return out


def get_app():
    ws = Quartz.NSWorkspace.sharedWorkspace()
    for app in ws.runningApplications():
        if "SpeedCrunch" in str(app.localizedName()):
            return app
    return None


class Session:
    def __init__(self):
        self.restart()

    def restart(self):
        subprocess.run(["pkill", "-x", "SpeedCrunch"], check=False)
        time.sleep(2)
        subprocess.run(["open", "-a", "SpeedCrunch"], check=False)
        time.sleep(4)
        app = get_app()
        app.activateWithOptions_(0)
        time.sleep(1)
        self.pid = app.processIdentifier()
        self.axapp = AXUIElementCreateApplication(self.pid)
        self.win = ax_copy(self.axapp, kAXMainWindowAttribute)
        self.refresh_refs()
        # A fresh launch restores the previous session's history: clear it
        # once so counts start from zero.
        self.clear_history()
        self.refresh_refs()

    def refresh_refs(self):
        self.win = ax_copy(self.axapp, kAXMainWindowAttribute)
        areas = find_textareas(self.win)
        # The history display sits above the input line: identify by screen
        # y, not by value length (both are empty on a fresh launch).
        self.inp = max(areas, key=lambda t: t[2])[0]
        self.hist = min(areas, key=lambda t: t[2])[0]

    def focus_input(self):
        AXUIElementSetAttributeValue(self.inp, kAXFocusedAttribute, True)
        time.sleep(0.05)

    def input_empty(self):
        return str(ax_copy(self.inp, kAXValueAttribute) or "") == ""

    def hist_value(self):
        return str(ax_copy(self.hist, kAXValueAttribute) or "")

    def hist_count(self):
        return len(value_lines(self.hist_value()))

    def clear_history(self):
        bar = ax_copy(self.axapp, kAXMenuBarAttribute)
        for item in ax_copy(bar, kAXChildrenAttribute) or []:
            if str(ax_copy(item, kAXTitleAttribute)) == "Edit":
                menus = ax_copy(item, kAXChildrenAttribute) or []
                for it in ax_copy(menus[0], kAXChildrenAttribute) or []:
                    if str(ax_copy(it, kAXTitleAttribute)) == "Clear History":
                        AXUIElementPerformAction(it, "AXPress")
                        time.sleep(0.3)
                        self._confirm_clear()
                        self.refresh_refs()
                        self.focus_input()
                        return True
        return False

    def _confirm_clear(self):
        for _ in range(5):
            wins = ax_copy(self.axapp, kAXWindowsAttribute) or []
            yes_btn = None
            for w in wins:
                if str(ax_copy(w, kAXTitleAttribute) or "").strip():
                    continue
                for k in ax_copy(w, kAXChildrenAttribute) or []:
                    if str(ax_copy(k, kAXRoleAttribute)) != "AXGroup":
                        continue
                    for b in ax_copy(k, kAXChildrenAttribute) or []:
                        if (
                            str(ax_copy(b, kAXRoleAttribute)) == "AXButton"
                            and str(ax_copy(b, kAXTitleAttribute)) == "Yes"
                        ):
                            yes_btn = b
            if yes_btn is None:
                return
            AXUIElementPerformAction(yes_btn, "AXPress")
            time.sleep(0.25)

    def set_clipboard(self, s):
        pb = NSPasteboard.generalPasteboard()
        pb.clearContents()
        pb.setString_forType_(s, NSPasteboardTypeString)

    def paste_chunk(self, lines, expect_count):
        """Paste lines into the empty focused input; wait until the history
        shows exactly expect_count results. Returns True on success."""
        for attempt in range(3):
            if not self.input_empty():
                self.clear_history()
                self.refresh_refs()
                self.focus_input()
            if not self.input_empty():
                time.sleep(0.3)
                continue
            self.set_clipboard("\n".join(lines))
            time.sleep(0.08)
            for down in (True, False):
                ev = Quartz.CGEventCreateKeyboardEvent(None, V_KEY, down)
                Quartz.CGEventSetFlags(ev, CMD)
                Quartz.CGEventPostToPid(self.pid, ev)
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                time.sleep(0.03)
                if self.hist_count() >= expect_count:
                    return True
            results_retry = None  # fall through to retry the chunk
            self.clear_history()
            self.refresh_refs()
            self.focus_input()
        return False


def value_lines(hist_text):
    """History text -> list of result strings, in entry order."""
    vals = []
    for m in re.finditer(r"^\s*=\s*(.+)$", hist_text, re.M):
        vals.append(m.group(1).strip())
    return vals


def parse_number(s):
    # SpeedCrunch renders the minus sign as U+2212 and groups digits with
    # spaces; normalize before float().
    return float(
        s.replace("\u2212", "-")
        .replace(" ", "")
        .replace("\u2009", "")
        .replace(",", "")
    )


def rel_ok(tool, referee, tol=1e-9):
    if referee == 0:
        return abs(tool) < tol
    return abs(tool - referee) / abs(referee) <= tol


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("task", choices=["w1", "w2"])
    ap.add_argument("--data", required=True)
    ap.add_argument("--acceptance", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    lines = [l.strip() for l in open(args.data) if l.strip()]
    acceptance = json.load(open(args.acceptance))
    ref = acceptance[args.task]
    n = len(lines)
    results = {
        "task": args.task,
        "label": "W1 scalar arithmetic" if args.task == "w1" else "W2 transcendental microbench",
        "chunk_lines": CHUNK,
        "gate": {
            "tolerance": "relative 1e-9 vs 60-digit decimal referee",
            "checked": 0,
            "failed": 0,
        },
        "retry": {"chunk_retries": 0},
    }

    sess = Session()

    # ---- verification pass (untimed; the correctness gate) ----
    checked = failed = 0
    i = 0
    while i < n:
        end = min(i + CHUNK, n)
        chunk = lines[i:end]
        if not sess.paste_chunk(chunk, end - i):
            results["retry"]["chunk_retries"] += 1
            results["gate"].setdefault("note", []).append(
                f"window {i}..{end - 1}: paste did not deliver after retries"
            )
            failed += end - i
            checked += end - i
            i = end
            sess.clear_history()
            sess.refresh_refs()
            continue
        time.sleep(0.1)
        vals = value_lines(sess.hist_value())
        if len(vals) != end - i:
            results["gate"].setdefault("note", []).append(
                f"window {i}..{end - 1}: expected {end - i} values, saw {len(vals)}"
            )
            failed += end - i
            checked += end - i
            i = end
        else:
            for k, v in enumerate(vals):
                checked += 1
                try:
                    if not rel_ok(parse_number(v), float(ref[i + k])):
                        failed += 1
                except ValueError:
                    failed += 1
                    results["gate"].setdefault("note", []).append(
                        f"window {i}..{end - 1}: unparsable value {v!r}"
                    )
            i = end
        sess.clear_history()
        sess.refresh_refs()
    results["gate"]["checked"] = checked
    results["gate"]["failed"] = failed

    if failed > 0:
        results["results"] = [
            {"verdict": "incorrect: no time published (plan rule 2)", "gate_failed": failed}
        ]
        json.dump(results, open(args.out, "w"), indent=1)
        print(json.dumps({"gate_failed": failed}))
        return 0

    # ---- timed passes: warm-up 3 untimed, then bucket samples ----
    def one_pass():
        """One full dataset pass in CHUNK-line pieces.

        Per-chunk span: Cmd+V posted -> the chunk's results present in the
        history value. Clears and clipboard writes are outside the span.
        Returns seconds, or None if a chunk failed all retries.
        """
        total = 0.0
        i = 0
        while i < n:
            end = min(i + CHUNK, n)
            if not sess.input_empty():
                sess.clear_history()
                sess.refresh_refs()
                sess.focus_input()
            else:
                sess.clear_history()
                sess.refresh_refs()
                sess.focus_input()
            sess.set_clipboard("\n".join(lines[i:end]))
            time.sleep(0.08)
            t0 = time.monotonic()
            for down in (True, False):
                ev = Quartz.CGEventCreateKeyboardEvent(None, V_KEY, down)
                Quartz.CGEventSetFlags(ev, CMD)
                Quartz.CGEventPostToPid(sess.pid, ev)
            deadline = time.monotonic() + 10
            delivered = False
            while time.monotonic() < deadline:
                time.sleep(0.03)
                if sess.hist_count() >= end - i:
                    total += time.monotonic() - t0
                    delivered = True
                    break
            if not delivered:
                results["retry"]["chunk_retries"] += 1
                sess.clear_history()
                sess.refresh_refs()
                sess.focus_input()
                # one wholesale retry of this chunk
                sess.set_clipboard("\n".join(lines[i:end]))
                time.sleep(0.08)
                t0 = time.monotonic()
                for down in (True, False):
                    ev = Quartz.CGEventCreateKeyboardEvent(None, V_KEY, down)
                    Quartz.CGEventSetFlags(ev, CMD)
                    Quartz.CGEventPostToPid(sess.pid, ev)
                deadline = time.monotonic() + 10
                while time.monotonic() < deadline:
                    time.sleep(0.03)
                    if sess.hist_count() >= end - i:
                        total += time.monotonic() - t0
                        delivered = True
                        break
                if not delivered:
                    return None
            i = end
        return total

    durations = []
    for _ in range(3):
        d = one_pass()
        while d is None:
            d = one_pass()
        durations.append(d)

    bucket = 30 if durations[0] < 1 else (7 if durations[0] < 60 else 3)
    samples = []
    for _ in range(bucket):
        d = one_pass()
        while d is None:
            results["retry"]["chunk_retries"] += 1
            d = one_pass()
        samples.append(d * 1000)
    samples.sort()
    med = (
        samples[len(samples) // 2]
        if len(samples) % 2
        else (samples[len(samples) // 2 - 1] + samples[len(samples) // 2]) / 2
    )
    mad = sorted(abs(s - med) for s in samples)[len(samples) // 2]
    p95 = samples[min(len(samples) - 1, max(0, int(0.95 * len(samples)) - 1))]
    results["results"] = [
        {
            "track": "single",
            "track_note": "W1/W2 fix the expressions; idiomatic and equivalent implementations coincide by construction (pilot README).",
            "boundary": "GUI bulk ingest end-to-end: per-chunk span is Cmd+V posted -> all of the chunk's results present in the history accessibility value; chunk spans summed; clears and clipboard writes outside the span",
            "warmups": 3,
            "requested_samples": bucket,
            "bucket_rule": f"warm-up pass {durations[0]:.1f} s -> {bucket} samples",
            "units": "ms",
            "samples": len(samples),
            "min": samples[0],
            "median": med,
            "mad": mad,
            "p95": p95,
            "max": samples[-1],
            "outlier_indices": [k for k, s in enumerate(samples) if s > 10 * med],
            "raw_passes_ms": samples,
        }
    ]
    json.dump(results, open(args.out, "w"), indent=1)
    print(json.dumps({"ok": True, "median_ms": med}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
