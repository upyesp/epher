# epher scripts

Ready-to-run scripts for the epher calculator, written by the maintainers and
by users. Every file here is a plain `.epher` script: the same format the
desktop app and web app save and open, and the CLI and REPL run.

`full-moons.epher` is the **reference example**: read it first, and follow it
when you contribute.

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

The language shapes the format: a script file runs **one statement per line**
(`;` joins statements on a line), there are no string literals, and comments
are PHP-style: `//` or `#` to the end of the line, `/* ... */` only when it
closes on the same line. The example turns those facts into a house style:

1. **Header comment first.** Name, one-line purpose, the algorithm and its
   published source, an honest accuracy statement, how to run the script,
   and what the user is expected to edit.
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
7. **Results last, labelled in comments.** There are no strings, so the
   comment above each output line is its label. Order outputs the way a
   reader wants them; one line, one answer.
8. **Show the expected output.** End with the shipped default's output as a
   comment, so anyone can check their build and their edit.
9. **Self-check when you can.** If a builtin can cross-check the script
   (like the ephemeris accessors), print the comparison.
10. **Honesty about accuracy.** State what is omitted and when the error
    grows. Never oversell.

## Contributing

- One script per file, named `lowercase-with-hyphens.epher`.
- Keep to the format standard above; the reference example is the bar.
- Run the script before opening a pull request and paste the expected
  output into its header comment. Scripts are checked against the current
  release, not a fork.
- Anything with a nontrivial algorithm cites its source; anything with an
  accuracy claim states its error.
- Contributions are licensed MIT, like the repo (see LICENSE).
