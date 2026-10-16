#!/usr/bin/env python3
"""Sync the shared Vim runtime files into the Neovim client.

The LuaRocks channel needs `clients/nvim` to be a self-contained Neovim
plugin directory: `lua/` for the module plus the runtimepath
directories the rock copies into its prefix. `clients/vim` carries the
one shared copy of the runtime files, used by Vim and by Neovim, so this
script copies the three runtime files into `clients/nvim` and keeps the
copies in lockstep. Edit `clients/vim/`, never the copies. Same rule as
the VS Code client's shared grammar and the Sublime grammar.

    python3 clients/nvim/sync-runtime.py          # write the copies
    python3 clients/nvim/sync-runtime.py --check  # fail on drift
"""

import argparse
import pathlib
import shutil
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
SOURCE = ROOT / "clients" / "vim"
TARGET = ROOT / "clients" / "nvim"

FILES = (
    "ftdetect/epher.vim",
    "ftplugin/epher.vim",
    "syntax/epher.vim",
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="write nothing and exit 1 when a copy differs from clients/vim",
    )
    args = parser.parse_args()

    stale = []
    for rel in FILES:
        source = SOURCE / rel
        target = TARGET / rel
        if args.check:
            if not target.exists() or target.read_bytes() != source.read_bytes():
                stale.append(rel)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        print(f"wrote {target.relative_to(ROOT)} from {source.relative_to(ROOT)}")

    if stale:
        print(
            "stale runtime copies (run clients/nvim/sync-runtime.py): "
            + ", ".join(stale),
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
