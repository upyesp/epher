//! gen-man — render `epher.1` (ADR-0013).
//!
//! Usage (from the repo root):
//!
//! ```sh
//! cargo run -p epher-cli --example gen-man > packaging/man/epher.1
//! ```
//!
//! clap_mangen renders the sections that mirror the argument surface
//! (NAME/SYNOPSIS/DESCRIPTION/OPTIONS/SUBCOMMANDS/VERSION — regenerated
//! whenever the CLI changes). The hand-written sections after it — the
//! language overview, shell commands, exit status, files, environment,
//! examples, and see-also — summarize the user guide
//! (site/guide/en.md); edit them there first, then here, then rerun.

use clap::CommandFactory;
use clap_mangen::roff::{roman, Roff};
use epher_cli::dispatch::Args;

fn main() {
    let mut cmd = Args::command();
    cmd.build();

    // The generated half: the sections that mirror the argument surface,
    // rendered individually so the order is canonical (man-pages(7)) and
    // nothing generic like clap's after-help text sneaks in.
    let man = clap_mangen::Man::new(cmd)
        .title("EPHER")
        .section("1")
        .manual("User Commands")
        .source("epher 0.5.1");
    let mut out = std::io::stdout();
    man.render_title(&mut out).expect("title");
    man.render_name_section(&mut out).expect("name");
    man.render_synopsis_section(&mut out).expect("synopsis");
    man.render_description_section(&mut out)
        .expect("description");
    man.render_options_section(&mut out).expect("options");
    man.render_subcommands_section(&mut out)
        .expect("subcommands");

    // The hand-written half, from the user guide.
    for section in sections() {
        section.to_writer(&mut out).expect("section");
    }
    man.render_version_section(&mut out).expect("version");
    see_also().to_writer(&mut out).expect("see also");
}

/// One custom man section: a heading plus roff content.
fn section(title: &str, build: impl FnOnce(&mut Roff)) -> Roff {
    let mut roff = Roff::default();
    roff.control("SH", [title]);
    build(&mut roff);
    roff
}

/// A `.TP` entry — a term with its description on the next line.
fn tagged(roff: &mut Roff, term: &str, desc: &str) {
    roff.control("TP", ["4"]);
    roff.text([roman(term)]);
    roff.text([roman(desc)]);
}

/// A paragraph of prose.
fn para(roff: &mut Roff, text: &str) {
    roff.text([roman(text)]);
}

fn sections() -> Vec<Roff> {
    let mut v = Vec::new();

    v.push(section("THE EPHER LANGUAGE", |roff| {
        para(roff, "epher evaluates arithmetic with the usual precedence: factorial (!) binds strongest, then power (^, right to left), then * and /, then + and -. Parentheses group. Numbers may use scientific notation (6.02e23). The constants pi, e, tau, and phi are always available.");
        para(roff, "Comparison and logic:");
        tagged(roff, "a > b, < >= <= == !=", "comparisons (true/false)");
        tagged(roff, "and, or, not", "logical connectives");
        para(roff, "Statements:");
        tagged(roff, "name = value", "assign a variable (x = 5)");
        tagged(roff, "const name = value", "define a constant (const tax = 0.2); immutable, visible everywhere, saved with `save`");
        tagged(roff, "if c then a else b", "conditional expression");
        tagged(roff, "while c do statement", "loop");
        tagged(roff, "def name(params) = expr", "define a function; recursion works (def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2))");
        tagged(roff, "stmt1; stmt2", "a script: statements joined with `;` or newlines (the same separator)");
        para(roff, "Exact layers (binary floats are exact here):");
        tagged(roff, "frac(n, d)", "exact fraction (frac(1, 3) = 1/3)");
        tagged(roff, "dec(x)", "exact decimal (dec(0.1) + dec(0.2) = 0.3)");
        tagged(roff, "big(x)", "exact whole number (big(10 ^ 20))");
    }));

    v.push(section("BUILT-IN FUNCTIONS", |roff| {
        para(
            roff,
            "Trigonometry works in radians; deg(x) and rad(x) convert.",
        );
        tagged(
            roff,
            "sin cos tan asin acos atan atan2",
            "trigonometric and inverse trigonometric",
        );
        tagged(roff, "deg(x), rad(x)", "radians to degrees and back");
        tagged(
            roff,
            "sinh cosh tanh asinh acosh atanh",
            "hyperbolic and inverse hyperbolic",
        );
        tagged(roff, "sqrt cbrt root(n, x) exp", "powers and roots");
        tagged(
            roff,
            "ln log log2 logb(b, x) hypot(a, b)",
            "logarithms and hypotenuse",
        );
        tagged(roff, "n! or fact(n)", "factorial");
        tagged(roff, "abs floor ceil round trunc sign", "rounding and sign");
        tagged(
            roff,
            "mod(a, b) gcd(a, b) lcm(a, b)",
            "remainders and common divisors",
        );
        tagged(
            roff,
            "ncr(n, r), npr(n, r)",
            "combinations and permutations",
        );
        tagged(
            roff,
            "sum product mean median min max variance stdev",
            "statistics over any number of arguments",
        );
    }));

    v.push(section("SHELL COMMANDS", |roff| {
        para(roff, "Inside the interactive session (repl) and piped scripts, these lines are shell commands, not expressions:");
        tagged(roff, "save name", "save the function or constant `name` for future sessions");
        tagged(roff, "save script name", "save the last evaluated line as a script");
        tagged(roff, "graph expr [from a to b]", "plot a curve (cartesian, param, polar; regions with y < / y >); the curves accumulate");
        tagged(roff, "graph3d z = f(x, y) [from a to b]", "plot a 3D surface");
        tagged(roff, "graph save file.svg | graph3d save file.svg", "write the current plot as a self-contained SVG image");
        tagged(roff, "graph clear | graph3d clear", "empty the plot");
        tagged(roff, "language code", "set the interface language: en, zh-CN, hi, es, fr, ar, de, pt");
        tagged(roff, "table expr [from a to b] [points n]", "print a table of values (TI-style defaults: -5..5, 11 rows)");
        tagged(roff, "quit, exit", "leave the interactive session (Ctrl-D too)");
    }));

    v.push(section("EXIT STATUS", |roff| {
        tagged(roff, "0", "success");
        tagged(roff, "1", "a calculation or runtime error");
        tagged(
            roff,
            "2",
            "usage error (bad arguments; also when `epher -` finds a terminal on stdin)",
        );
    }));

    v.push(section("FILES", |roff| {
        tagged(roff, "~/.epher/", "the epher store: saved functions, constants, scripts, history, and the language preference. Delete it to start fresh.");
    }));

    v.push(section("ENVIRONMENT", |roff| {
        tagged(
            roff,
            "EPHER_STORE_DIR",
            "relocate the store (EPHER_STORE_DIR=/tmp/scratch epher repl)",
        );
        tagged(
            roff,
            "NO_COLOR",
            "disable colored error output when set (see no-color.org)",
        );
    }));

    v.push(section("EXAMPLES", |roff| {
        tagged(roff, "epher \"2 + 3 * 4\"", "print the value of an expression: 14");
        tagged(roff, "epher \"x = 10; x + 5\"", "scripts work too: each statement's value prints (10 then 15)");
        tagged(roff, "epher \"-2 + 5\"", "a leading minus is part of the expression; prints 3");
        tagged(roff, "printf \"x = 3\\nx * 10\\n\" | epher -", "read a script from standard input, line by line");
        tagged(roff, "printf \"def f(x) = x ^ 2\\nf(9)\\n\" | epher -", "lines share one session: prints = 81");
        tagged(roff, "epher repl", "interactive session; answers print as `= result`");
        tagged(roff, "epher tui", "full-screen interface with graphing (graph x ^ 2, graph3d x ^ 2 - y ^ 2, space animates)");
    }));

    v
}

fn see_also() -> Roff {
    let mut roff = Roff::default();
    roff.control("SH", ["SEE ALSO"]);
    para(
        &mut roff,
        "The user guide, in eight languages, and the project itself:",
    );
    tagged(&mut roff, "https://epher.org/guide/", "the full user guide");
    tagged(
        &mut roff,
        "https://github.com/upyesp/epher",
        "source code, releases, and the issue tracker",
    );
    roff
}
