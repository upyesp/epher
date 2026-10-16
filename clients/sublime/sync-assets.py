#!/usr/bin/env python3
"""Sync the shared editor assets into the Sublime package.

Sublime Text loads TextMate grammars in the plist XML form
(`.tmLanguage`), not the JSON form the other clients share, so this
script converts `clients/shared/epher.tmLanguage.json` into
`clients/sublime/epher.tmLanguage` and commits it. Run it after
editing the shared grammar; never edit the generated file.

    python3 clients/sublime/sync-assets.py
"""

import json
import plistlib
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
SOURCE = ROOT / "clients" / "shared" / "epher.tmLanguage.json"
TARGET = ROOT / "clients" / "sublime" / "epher.tmLanguage"


def main() -> None:
    with SOURCE.open(encoding="utf-8") as f:
        grammar = json.load(f)

    with TARGET.open("wb") as f:
        plistlib.dump(grammar, f)

    print(f"wrote {TARGET.relative_to(ROOT)} from {SOURCE.relative_to(ROOT)}")


if __name__ == "__main__":
    # Sublime Text imports every top-level module of an installed
    # package as a plugin; this file must stay import-safe there.
    main()
