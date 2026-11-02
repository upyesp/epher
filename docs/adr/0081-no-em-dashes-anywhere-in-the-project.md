# ADR-0081: no em-dashes anywhere in the project

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-02

## Context

The project's writing rule bans the em-dash, but until now the rule
lived only in conversation. It kept coming back: the accessibility
page (November 2026) shipped with em-dashes in its English copy and
in several of the eight translations, and a follow-up sweep found more
in older site strings (`mac-dmg-hint`, `linux-deb-hint`, two IDE page
strings), the guide and reference sources, every `docs/research` note,
most ADRs, client store listings, and CI workflow comments. Over
seven hundred occurrences were removed across those files in the
sweep that produced this ADR.

Two things make the character worth a written rule rather than a
preference. First, a flat ban is checkable: a script can find U+2014
exactly, while "use dashes sparingly" cannot be enforced. Second, the
em-dash has no clean equivalent in several of the eight interface
languages (Chinese renders it doubled, Arabic and Hindi wrap it
differently), so English copy built on em-dash asides translates
badly; a ban at the source keeps all eight catalogs structurally
parallel.

## Decision

The em-dash (U+2014), including the doubled `——`, is banned in every
project-authored text: website copy and its translations, the guide
and reference, `docs/` (ADRs, research notes, audits), code comments,
commit messages, release notes, store listings, and CI summaries.
Replacement punctuation: a semicolon separates independent clauses,
parentheses hold asides, a colon introduces elaboration. Ranges and
compound names take the hyphen. The math minus sign (U+2212) is a
different character and is unaffected, including inside plot and
terminal output.

Scope boundary: vendored third-party code (`vendor/`) is out of scope;
its text follows upstream. Existing ADRs and research notes were
normalized in the sweep that landed this ADR; that pass was strictly
typographic and changed no decision, which is within the README's
immutability rule (it protects decision content, not spelling).

## Consequences

`scripts/a11y/site-audit.mjs` enforces the ban on every deploy: it
scans everything served under `site/` (all pages, the generated guide
and reference HTML, their markdown sources, and all eight translation
catalogs) and fails CI on a single occurrence. The sweep also cleaned
the audit scripts' own output strings, since those print into CI logs.

Code-bearing directories (`crates/`, `clients/` sources, `vendor/`)
are not scanned by the site audit; their comments and user-facing
strings normalize through their normal trains. Repository history is
not rewritten; commit messages simply drop the character from this
ADR onward.

## Alternatives considered

Per-locale style guides that allow the dash where typographically
traditional were rejected: the value of the rule is that a script can
enforce it, and judgment calls defeat that. Linting every tracked
file, including code and vendor trees, was rejected for now: those
changes belong to code trains, and the text readers actually meet is
the served site, which the audit covers completely.
