#!/usr/bin/env python3
"""Update guide text blocks for ADR-0052 (script answers + parity fixes).

String-based: locate each fence verbatim, then replace the content of
the text block that immediately follows it. Blocks stay byte-identical
across the 8 locales; fences are untouched.
"""
import sys

FENCES = {
    "price": "```epher\nprice = 100\nif price > 50 then 2 else 1\n```",
    "while": "```epher\nx = 0; while x < 5 do x = x + 1; x\n```",
    "semicolon": "```epher\nx = 10; y = x + 5; x + y\n```",
    "constk": "```epher\nconst k = 3\nsolve k*x == 12\n```",
    "ttest": "```epher\nd = {12, 15, 14, 16, 13, 15, 14, 17}\nttest(d, 14)\ntinterval(d, 0.95)\nztest(d, 14, 1.5)\nchisq_gof({20, 30, 25, 25}, {25, 25, 25, 25})\n```",
    "tvm": "```epher\ntvm_pmt(360, 0.08/12, -100000, 0)\n```",
    "equinox": "```epher\nmarch_equinox(2000)\n```",
    "sqrtneg": "```epher\nsqrt(-4)\n```",
    "grouped": "```epher\nscientific(12345)\nengineering(12345)\nengineering(0.5)\ngrouped(1234567.89)\n```",
}

NEW_BLOCKS = {
    "price": "100\n2",
    "while": "0\n5",
    "semicolon": "10\n15\n25",
    "constk": "3\nx = 4",
    "ttest": "{12, 15, 14, 16, 13, 15, 14, 17}\nt = 0.8819, p = 0.4071\n(13.1594, 15.8406)\nz = 0.9428, p = 0.3458\nchi2 = 2, p = 0.5724",
    "tvm": "733.764573879",
    "equinox": "1012520636/413",
    "sqrtneg": "2i",
    "grouped": "1.2345e4\n12.345e3\n500e-3\n1\u2009234\u2009567.89",
}

LANGS = ["en", "zh-CN", "hi", "es", "fr", "ar", "de", "pt"]
changed = 0
for lang in LANGS:
    path = f"site/guide/{lang}.md"
    text = open(path).read()
    for key, fence in FENCES.items():
        idx = text.find(fence)
        if idx == -1:
            print(f"{lang}: {key}: fence not found!")
            continue
        rest = text[idx + len(fence):]
        # the fence is followed by blank lines then ```text ... ```
        m = None
        import re
        m = re.match(r"\n+```text\n(.*?)\n```", rest, re.S)
        if not m:
            print(f"{lang}: {key}: no text block after fence!")
            continue
        old_block = m.group(1)
        new_block = NEW_BLOCKS[key]
        if old_block == new_block:
            continue
        text = text[: idx + len(fence)] + rest[: m.start(1)] + new_block + rest[m.end(1):]
        changed += 1
    open(path, "w").write(text)
print(f"updated {changed} blocks across {len(LANGS)} locales")
