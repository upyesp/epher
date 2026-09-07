# -*- coding: utf-8 -*-
"""Round 3 guide translation: inserts the translated 1.20-1.22 sections,
quick-reference rows, and table paragraphs into the 7 non-English guides.
Fences are reused VERBATIM from en.md so they stay byte-identical."""
import io, re

EN = io.open("site/guide/en.md", encoding="utf-8").read()

def fence(label, kind="epher"):
    for m in re.finditer(r"```%s\n([^`]*?)```" % kind, EN, re.S):
        if label in m.group(1):
            return m.group(0)
    raise SystemExit("missing fence: " + label)

F = {
    "probe_list": fence("d[2]"),
    "arith": fence("{1, 2, 3} * 2"),
    "quart": fence("quartile(d, 1)"),
    "linreg": fence("linreg({1, 2, 3, 4}"),
    "norm": fence("normcdf(1.96)"),
    "tests": fence("ttest(d, 14)"),
    "scatter": fence("graph scatter(x, y)"),
    "histogram": fence("graph histogram("),
    "boxplot": fence("graph boxplot("),
}
F2_TABLE = fence("derivative x ^ 2", "epher")
F2_TABLE_OUT = fence("y'", "text")
assert F2_TABLE and F2_TABLE_OUT
assert "table" in F2_TABLE and "derivative" in F2_TABLE

ANCHORS = {
    "zh-CN": "## 2. 网页应用（PWA）",
    "hi": "## 2. वेब ऐप (PWA)",
    "es": "## 2. La aplicación web (PWA)",
    "fr": "## 2. L'application web (PWA)",
    "ar": "## 2. تطبيق الويب (PWA)",
    "de": "## 2. Die Web-App (PWA)",
    "pt": "## 2. A aplicação web (PWA)",
}

SECTIONS = {}
exec(io.open("scripts/round3-sections.py", encoding="utf-8").read(), {"SECTIONS": SECTIONS, "F": F})
QR = {}
TABLE_PARA = {}
TABLE_CELLS = {}
exec(io.open("scripts/round3-rows.py", encoding="utf-8").read(), {"QR": QR, "TABLE_PARA": TABLE_PARA, "TABLE_CELLS": TABLE_CELLS})

def apply(loc, path):
    md = io.open(path, encoding="utf-8").read()
    idx = md.index(ANCHORS[loc])
    section = SECTIONS[loc]
    for tok, fence in F.items():
        section = section.replace("%%TOKEN%%".replace("TOKEN", tok.upper()), fence)
    md = md[:idx] + section + "\n" + md[idx:]

    marker = "| `factors(360)` |"
    assert marker in md, loc
    md = md.replace(marker, marker + "\n" + "\n".join(QR[loc]), 1)

    # The derivative-column example is NEW: insert it (prose + both
    # fences) right before each locale's existing plain table example.
    plain_fence = "```epher\ntable x ^ 2 from -2 to 2 points 5\n```"
    assert plain_fence in md, (loc, "plain table fence")
    block = (
        TABLE_PARA[loc]
        + "\n\n"
        + F2_TABLE
        + "\n\n"
        + F2_TABLE_OUT
        + "\n\n"
        + TABLE_CELLS[loc]
        + "\n"
    )
    md = md.replace(plain_fence, block + plain_fence, 1)

    io.open(path, "w", encoding="utf-8").write(md)
    print(loc, "done")

for loc in ANCHORS:
    apply(loc, "site/guide/%s.md" % loc)

# ---- verification: fences byte-identical across locales ----
def epher_fences(md):
    return re.findall(r"```epher\n.*?```", md, re.S)

en_f = epher_fences(io.open("site/guide/en.md", encoding="utf-8").read())
print("en epher fences:", len(en_f))
ok = True
for loc in ANCHORS:
    md = io.open("site/guide/%s.md" % loc, encoding="utf-8").read()
    loc_f = epher_fences(md)
    if len(loc_f) != len(en_f):
        print("COUNT MISMATCH", loc, len(loc_f), "vs", len(en_f))
        ok = False
        continue
    for i, (a, b) in enumerate(zip(en_f, loc_f)):
        if a != b:
            print("FENCE MISMATCH", loc, "index", i)
            print("  en:", a[:60].replace("\n", "\\n"))
            print("  " + loc + ":", b[:60].replace("\n", "\\n"))
            ok = False
    print(loc, "fences:", len(loc_f), "identical" if ok else "DIFF")
print("ALL IDENTICAL" if ok else "FAILED")
