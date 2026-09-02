# epher scripts

Ready-to-run scripts for the epher calculator, written by the maintainers and
by users. Every file here is a plain `.epher` script: the same format the
desktop app and web app save and open, and the CLI and REPL run.

`full-moons.epher` is the **reference example**: read it first, and follow it
when you contribute. It shows the language as it stands today: plain
arithmetic and `def` helpers, the newer `for`, `if`, `print` and strings
(guide 1.7-1.9 and 1.29-1.30), and the astronomy accessors and constants
(guide 1.16). Each use names the guide chapter that teaches it, so a reader
can look any line up.

## Running a script

From a terminal (the CLI runs every line in order and prints each result):

```sh
epher "epher scripts/full-moons.epher"
```

In the REPL, `load` takes a file path or a script saved with `save script`:

```text
epher> load epher scripts/full-moons.epher
```

In the web or desktop app, paste the whole file into the entry and press
Enter (Shift+Enter for new lines). Everything also works line by line in the
TUI. See the guide, chapter 4, for scripts, `save` and `load`.

## The format standard

The language shapes the format: a script file runs **one statement per
line** (`;` joins statements on a line), comments are PHP-style (`//` or `#`
to the end of the line, `/* ... */` on one line), and every statement's
value shows as epher displays it, exact fractions included. The example
turns those facts into a house style:

1. **Header comment first.** Name, one-line purpose, the algorithm and its
   published source, an honest accuracy statement, how to run the script,
   what the user is expected to edit, and what the file demonstrates.
2. **Knobs at the top.** Every value a user might change is one `const` in a
   clearly marked block right after the header. Nothing to edit below it.
3. **Cite your source.** Name the book, paper, or data source in the header
   and name each equation or table at its section divider.
4. **Section dividers.** A `// ---- label ----` line between sections:
   knob, elements/model, helpers, results.
5. **One job per helper.** Short lowercase `def` names (`mp`, `corr_a`,
   `kfirst`), each with a trailing `//` note saying what it is. Helpers are
   defined before first use.
6. **Units in the code.** Write `30 deg`, `1 AU`, `24 hr`, convert with
   `in`, and say the unit in the trailing comment when a number is a count.
7. **Results last, labelled.** `print("label:", value)` (guide 1.29) puts a
   readable label on the same line as its value; when a result speaks for
   itself, leave it bare so it keeps epher's native display. A comment above
   each result says what it is and why it is there. Order outputs the way a
   reader wants them; one line, one answer.
8. **Show the expected output.** End with the shipped default's output as a
   comment, so anyone can check their build and their edit.
9. **Self-check when you can.** If a builtin can cross-check the script
   (like the ephemeris accessors), print the comparison.
10. **Honesty about accuracy.** State what is omitted and when the error
    grows. Never oversell. The example's illum numbers are rounded by the
    builtin to four decimals; say so when it matters.

## Contributing

- One script per file, named `lowercase-with-hyphens.epher`.
- Keep to the format standard above; the reference example is the bar.
- Run the script before opening a pull request and paste the expected
  output into its header comment. Scripts are checked against the current
  release, not a fork.
- Anything with a nontrivial algorithm cites its source; anything with an
  accuracy claim states its error.
- Contributions are licensed MIT, like the repo (see LICENSE).
