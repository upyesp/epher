# tree-sitter-epher

The epher grammar for [tree-sitter](https://tree-sitter.github.io):
statements, expressions, quantities with unit suffixes, and the three
comment forms, written from the language reference in the
[epher repository](https://github.com/upyesp/epher).

## What the grammar covers

- all statement forms: assignment, `const`, destructuring
  (`{a, b} =`), both `def` body forms, `return`/`break`/`continue`,
  statement `if`/`while`/`for`, `solve`;
- expressions with the reference's exact precedence ladder, including
  the conditional expression `if c then a else b`, ranges
  (`1 to 5 step 2`), unit conversion (`in`, `->`), factorial and
  percent postfixes, indexing, calls, strings with the five escapes,
  lists, and matrices;
- quantities (`2 m`, `60 mile/hr`, `2m`): the unit suffix is scanned
  by a small external scanner (`src/scanner.c`) because the rule is
  adjacency-sensitive - same line as the number, spaces allowed, a
  newline never. Reserved words never become units, so `1 to 5` stays
  a range and `2 in cm` stays a conversion;
- numbers in every spelling: decimal, scientific, `0b`/`0o`/`0x`,
  imaginary `i`.

Known looseness: comparisons are accepted chained (the reference says
they do not chain). A highlighting grammar parses a superset; the
language server is the truth.

## Build and test

```
tree-sitter generate
tree-sitter test
```

The grammar is verified against the whole `epher scripts` corpus: all
433 shipped scripts parse without errors.

## Used by

- the [epher Zed extension](https://github.com/upyesp/epher/tree/main/clients/zed)
  (pinned by revision in its `extension.toml`);
- anything else that wants epher highlighting through tree-sitter.

Versioning rides epher's 0.5.x train (ADR-0066).
