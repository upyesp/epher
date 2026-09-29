#!/usr/bin/env python3
"""Sync the shared grammar files into the Eclipse bundle.

The Eclipse Marketplace channel needs `clients/eclipse` to be a
self-contained Tycho build, and `build.properties` bundles
`syntaxes/epher.tmLanguage.json` and `language-configuration.json`
straight from the bundle directory. `clients/shared` and
`clients/vscode` carry the one shared copy of each file, used by every
client, so this script copies them into the bundle and keeps the copies
in lockstep. Edit the shared sources, never the copies. Same rule as
the Neovim runtime copies and the Sublime grammar.

    python3 clients/eclipse/sync-assets.py          # write the copies
    python3 clients/eclipse/sync-assets.py --check  # fail on drift
"""

import argparse
import pathlib
import shutil
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
BUNDLE = ROOT / "clients" / "eclipse" / "io.github.upyesp.epher.eclipse"

# (shared source, path inside the bundle)
FILES = (
    ("clients/shared/epher.tmLanguage.json", "syntaxes/epher.tmLanguage.json"),
    ("clients/vscode/language-configuration.json", "language-configuration.json"),
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="write nothing and exit 1 when a copy differs from its source",
    )
    args = parser.parse_args()

    stale = []
    for source_rel, target_rel in FILES:
        source = ROOT / source_rel
        target = BUNDLE / target_rel
        if args.check:
            if not target.exists() or target.read_bytes() != source.read_bytes():
                stale.append(target_rel)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        print(f"wrote {target.relative_to(ROOT)} from {source_rel}")

    if stale:
        print(
            "stale eclipse bundle copies (run clients/eclipse/sync-assets.py): "
            + ", ".join(stale),
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
