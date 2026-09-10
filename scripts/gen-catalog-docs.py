#!/usr/bin/env python3
"""Generate catalog docs from the language reference (one-off, ADR-0066).

The reference (site/reference.md) documents every callable and constant
with a signature and a one-line meaning, and the drift-guard test keeps
those names synced with epher-core. This script lifts that text into the
catalog entries themselves, so hover in editors says exactly what the
reference says. Run once from the repo root:
    python3 scripts/gen-catalog-docs.py
"""

import re
import sys

REF = "site/reference.md"
LIB = "crates/core/src/lib.rs"

md = open(REF, encoding="utf-8").read()

# --- parse the reference tables -------------------------------------------
section = None
constants = {}   # name -> meaning
functions = {}   # name -> (signature, description)

for line in md.splitlines():
    m = re.match(r"^### (\d+\.\d+) ", line)
    if m:
        section = m.group(1)
        continue
    if not line.startswith("|") or section is None:
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) < 2 or not cells[0].startswith("`"):
        continue
    if section.startswith("9."):
        # | `name` | value | meaning |
        if len(cells) != 3:
            continue
        name = cells[0].strip("`")
        if name == "Constant":
            continue
        constants[name] = cells[2]
    elif section.startswith("10."):
        # | `sig`, `sig` | description |  (one row may carry siblings)
        if len(cells) != 2:
            continue
        sigs = re.findall(r"`([^`]+)`", cells[0])
        desc = cells[1]
        for sig in sigs:
            name = sig.split("(", 1)[0]
            if not name or not name.replace("_", "a").isalnum():
                continue
            if name in functions:
                s0, d0 = functions[name]
                sigs_all = s0 if sig in s0 else f"{s0} / {sig}"
                descs = d0 if desc in d0 else f"{d0}; {desc}"
                functions[name] = (sigs_all, descs)
            else:
                functions[name] = (sig, desc)

# --- house style: no em-dashes in in-app copy (CONTEXT.md) ---------------
def clean(s):
    s = s.replace(" \u2014 ", ": ").replace("\u2014", "-")
    return s.replace("\\", "\\\\").replace('"', '\\"')

# --- the catalog as it stands ---------------------------------------------
lib = open(LIB, encoding="utf-8").read()
block_re = re.compile(
    r"static BUILTIN_CATALOG: &\[CatalogEntry\] = &\[(.*?)\n\];", re.S)
block = block_re.search(lib)
if not block:
    sys.exit("BUILTIN_CATALOG block not found")
entries = re.findall(
    r'CatalogEntry \{\s*name: "([^"]+)",\s*kind: CatalogKind::(\w+),\s*\}',
    block.group(1),
)
print(f"catalog entries: {len(entries)}")

def docs_for(name, kind):
    if kind == "Constant":
        if name not in constants:
            sys.exit(f"no reference meaning for constant {name}")
        return (name, constants[name])
    if name not in functions:
        sys.exit(f"no reference signature for function {name}")
    return functions[name]

out = []
missing = []
for name, kind in entries:
    if (name, kind) not in [(n, k) for n, k in entries]:
        missing.append(name)
    sig, desc = docs_for(name, kind)
    out.append(
        "    CatalogEntry {\n"
        f'        name: "{name}",\n'
        f"        kind: CatalogKind::{kind},\n"
        f'        signature: "{clean(sig)}",\n'
        f'        description: "{clean(desc)}",\n'
        "    },"
    )

catalog_names = {n for n, _ in entries}
supp = [n for n in sorted(functions) if n not in catalog_names]
supp_out = []
for name in supp:
    sig, desc = functions[name]
    supp_out.append(
        "    CatalogEntry {\n"
        f'        name: "{name}",\n'
        "        kind: CatalogKind::Function,\n"
        f'        signature: "{clean(sig)}",\n'
        f'        description: "{clean(desc)}",\n'
        "    },"
    )
print(f"supplementary (documented, uncataloged): {len(supp)}: {', '.join(supp)}")

new_static = "static BUILTIN_CATALOG: &[CatalogEntry] = &[\n" + "\n".join(out) + "\n];\n"
if supp_out:
    new_static += (
        "\n/// Callables the reference documents but the catalog does not\n"
        "/// list (the astronomy family, `print`, the calculus specials):\n"
        "/// hover and completion fuel for names users type but the data\n"
        "/// keypad banks never suggested (ADR-0066). Membership of\n"
        "/// `catalog()` itself is unchanged.\n"
        "static SUPPLEMENTARY_CATALOG: &[CatalogEntry] = &[\n"
        + "\n".join(supp_out)
        + "\n];\n"
        "\n/// The supplementary callables (ADR-0066): documented names\n"
        "/// outside the curated catalog.\n"
        "pub fn supplementary_catalog() -> &'static [CatalogEntry] {\n"
        "    SUPPLEMENTARY_CATALOG\n"
        "}\n"
    )
lib = lib[: block.start()] + new_static + lib[block.end():]
open(LIB, "w", encoding="utf-8").write(lib)
print("lib.rs rewritten")
