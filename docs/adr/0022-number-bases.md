# ADR-0022: Number bases — `0b`/`0o`/`0x` literals and `bin`/`oct`/`hex`

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

A programmable calculator without binary/octal/hex is a handicap for
anyone doing bit arithmetic, networking, or just double-checking
conversions. The request: support number bases "with the correct
notation as recognised by the math community" — the C-family/Python
prefix spellings `0b101`, `0o17`, `0xFF`, not calculator-specific
subscript or `16#FF#` notations.

## Decision

- **Literals:** the tokenizer accepts `0b`/`0o`/`0x` (either case of the
  letter) followed by the digits the base admits — `0b101` is 5,
  `0o17` is 15, `0xFF` is 255. A prefix changes the spelling, never the
  value: `0xFF + 0b1` is 256, exactly like `255 + 1`. Like decimal
  literals, the token is an `f64` (exact up to 2^53). `0b2` is a parse
  error naming the bad digit; `0x` alone asks for digits.
- **Conversion:** `bin(x)`, `oct(x)`, `hex(x)` take one whole number
  and return a *string* — the prefixed spelling (`bin(10)` → `0b1010`),
  negative sign on the prefix (`hex(-42)` → `-0x2a`), lowercase digits.
  The answer feeds straight back in as a literal. All exact layers
  convert: `Float` (if integral), `Rational` (denominator 1), `Decimal`
  (integral), `Big`; fractions and non-numbers are type errors.
- **A `Str` variant joins `Value`** to carry these strings. It displays
  as itself and has no operations — the language still has no string
  literals, concatenation, or comparison. `ans` records it like any
  value.
- Keypads (web `nΣ` bank, TUI `num` bank) gain `bin`/`oct`/`hex` keys.
- Guide §1.12 explains the bases, §1.13 and §1.15 list the functions
  in all eight locales (audit-fenced identically).

## Consequences

- Community-standard spellings, no new punctuation class in the
  tokenizer (the `0b`-style branch is one peekable-match arm).
- `0x` (formerly a parse error as `0` followed by a name) is now an
  explicit "expected digits" error — strictly more informative.
- Base-conversion output is display-only by design; arithmetic on it
  fails with a type error rather than a silent coercion.
