#!/usr/bin/env python3
"""Convert clients/shared/epher.tmLanguage.json to plist XML.

The JetBrains TextMate plugin reads the classic plist form, so the
grammar the plugin ships under epher-bundle/syntaxes/epher.tmLanguage
is generated from the shared JSON grammar with this script:

    python3 clients/jetbrains/scripts/json2plist.py \
        clients/shared/epher.tmLanguage.json \
        clients/jetbrains/src/main/resources/epher-bundle/syntaxes/epher.tmLanguage

Mappings: JSON objects become <dict>, arrays become <array>, strings
become <string>, booleans become <true/>/<false>, integers become
<integer>, and floats become <real>. Key order is preserved.
"""

import json
import sys
from xml.sax.saxutils import escape

INDENT = "\t"


def convert(value, depth: int) -> str:
    pad = INDENT * depth
    inner = INDENT * (depth + 1)
    if isinstance(value, dict):
        lines = [f"{pad}<dict>"]
        for key, item in value.items():
            lines.append(f"{inner}<key>{escape(str(key))}</key>")
            lines.append(convert(item, depth + 1))
        lines.append(f"{pad}</dict>")
        return "\n".join(lines)
    if isinstance(value, list):
        lines = [f"{pad}<array>"]
        for item in value:
            lines.append(convert(item, depth + 1))
        lines.append(f"{pad}</array>")
        return "\n".join(lines)
    if isinstance(value, bool):
        # bool must be checked before int: bool subclasses int
        return f"{pad}<true/>" if value else f"{pad}<false/>"
    if isinstance(value, int):
        return f"{pad}<integer>{value}</integer>"
    if isinstance(value, float):
        return f"{pad}<real>{value}</real>"
    return f"{pad}<string>{escape(str(value))}</string>"


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit("usage: json2plist.py <input.json> <output.tmLanguage>")
    source, target = sys.argv[1], sys.argv[2]
    with open(source, encoding="utf-8") as handle:
        grammar = json.load(handle)
    body = convert(grammar, 1)
    document = (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\""
        " \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n"
        '<plist version="1.0">\n'
        f"{body}\n"
        "</plist>\n"
    )
    with open(target, "w", encoding="utf-8") as handle:
        handle.write(document)
    print(f"{target}: {len(document)} bytes")


if __name__ == "__main__":
    main()
