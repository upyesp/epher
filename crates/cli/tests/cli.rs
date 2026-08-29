use std::io::Write;
use std::process::{Command, Stdio};

fn epher_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_epher-cli"))
}

/// Run a REPL session with piped stdin, returning its stdout and stderr.
fn repl_session(store_dir: &str, input: &str) -> (String, String) {
    let mut child = epher_bin()
        .arg("repl")
        .env("EPHER_STORE_DIR", store_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// Run a REPL session with piped stdin, returning its stdout.
fn repl_output(store_dir: &str, input: &str) -> String {
    repl_session(store_dir, input).0
}

/// Run one-shot against an isolated store dir, returning stdout.
fn one_shot_output(expr: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let out = epher_bin()
        .arg(expr)
        .env("EPHER_STORE_DIR", dir.path().to_str().unwrap())
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn one_shot_evaluates_and_prints() {
    assert_eq!(one_shot_output("2 + 3 * 4").trim(), "14");
}

#[test]
fn one_shot_accepts_semicolon_scripts() {
    assert_eq!(one_shot_output("x = 10; x + 5").trim(), "10\n15");
}

#[test]
fn one_shot_accepts_newline_separated_scripts() {
    assert_eq!(one_shot_output("x = 3\nx * 10").trim(), "3\n30");
}

#[test]
fn one_shot_accepts_mixed_separators_and_blank_lines() {
    assert_eq!(one_shot_output("x = 3;;\n\ny = x + 1\ny").trim(), "3\n4\n4");
}

#[test]
fn one_shot_errors_on_bad_input() {
    let dir = tempfile::tempdir().unwrap();
    let out = epher_bin()
        .arg("2 +")
        .env("EPHER_STORE_DIR", dir.path().to_str().unwrap())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("error"));
}

#[test]
fn repl_runs_scripts_and_keeps_state() {
    let dir = tempfile::tempdir().unwrap();
    let out = repl_output(
        dir.path().to_str().unwrap(),
        "x = 5; x + 1\ndef f(n) = n * 2\nf(x)\nquit\n",
    );
    assert!(out.contains("6"), "stdout was: {out}");
    assert!(out.contains("10"), "stdout was: {out}");
    // the bare def produces no error line
    assert!(!out.contains("error"), "stdout was: {out}");
}

#[test]
fn repl_persists_functions_and_history_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // session 1: define + save a function, evaluate, quit
    let out1 = repl_output(path, "def f(x) = x ^ 2\nsave f\nf(3)\nquit\n");
    assert!(out1.contains("saved f"), "stdout was: {out1}");
    assert!(out1.contains("= 9"), "stdout was: {out1}");

    // session 2: the saved function is loaded from the store
    let out2 = repl_output(path, "f(4)\nquit\n");
    assert!(out2.contains("= 16"), "stdout was: {out2}");

    // history persisted too (visible as the definition line on load? no —
    // history is display-only; check the store file exists)
    assert!(dir.path().join("function/f.json").exists());
    assert!(dir.path().join("setting/history.json").exists());
}

#[test]
fn repl_save_requires_a_definition_in_session() {
    let dir = tempfile::tempdir().unwrap();
    let (out, err) = repl_session(dir.path().to_str().unwrap(), "save nope\nquit\n");
    assert!(err.contains("no definition for nope"), "stderr was: {err}");
    assert!(!out.contains("no definition"), "stdout was: {out}");
}

#[test]
fn language_command_persists_the_setting() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let out = repl_output(path, "language fr\nquit\n");
    assert!(out.contains("language set to fr"), "stdout was: {out}");
    // the preference is stored and reloaded on restart
    assert!(dir.path().join("setting/language.json").exists());
    let raw = std::fs::read_to_string(dir.path().join("setting/language.json")).unwrap();
    assert!(raw.contains("\"fr\""), "setting file was: {raw}");

    let out2 = repl_output(path, "quit\n");
    assert!(out2.contains("epher>"), "stdout was: {out2}");
}

#[test]
fn unsupported_language_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let (out, err) = repl_session(dir.path().to_str().unwrap(), "language xx\nquit\n");
    assert!(err.contains("unsupported language xx"), "stderr was: {err}");
    assert!(!out.contains("unsupported language"), "stdout was: {out}");
}

#[test]
fn save_script_persists_and_reloads_the_last_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // session 1: run a script, save it
    let out1 = repl_output(path, "x = 10; y = x + 5\nsave script setup\nquit\n");
    assert!(out1.contains("saved script setup"), "stdout was: {out1}");
    assert!(dir.path().join("script/setup.json").exists());

    // session 2: the saved script ran at startup (y is defined)
    let out2 = repl_output(path, "y\nquit\n");
    assert!(out2.contains("= 15"), "stdout was: {out2}");
}

#[test]
fn save_script_without_a_preceding_line_errors() {
    let dir = tempfile::tempdir().unwrap();
    let (out, err) = repl_session(dir.path().to_str().unwrap(), "save script empty\nquit\n");
    assert!(err.contains("nothing to save"), "stderr was: {err}");
    assert!(!out.contains("nothing to save"), "stdout was: {out}");
}

/// The clig.dev conventions (ADR-0013): stdout carries data, stderr
/// carries diagnostics, and exit codes tell scripts what happened.
mod conventions {
    use super::*;

    #[test]
    fn stdin_script_prints_results_to_stdout_and_errors_to_stderr() {
        let dir = tempfile::tempdir().unwrap();
        let mut child = epher_bin()
            .arg("-")
            .env("EPHER_STORE_DIR", dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"2 + 2\n1 / 0\n3 * 3\n")
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stdout.contains("= 4"), "stdout was: {stdout}");
        assert!(stdout.contains("= 9"), "stdout was: {stdout}");
        assert!(
            !stdout.contains("error"),
            "errors must not pollute piped data: {stdout}"
        );
        assert!(
            stderr.contains("error: division by zero"),
            "stderr was: {stderr}"
        );
        // the script kept going *and* reported failure
        assert_eq!(out.status.code(), Some(1));
    }

    #[test]
    fn clean_stdin_script_exits_zero() {
        let dir = tempfile::tempdir().unwrap();
        let mut child = epher_bin()
            .arg("-")
            .env("EPHER_STORE_DIR", dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.as_mut().unwrap().write_all(b"2 + 2\n").unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&out.stderr).is_empty());
    }

    #[test]
    fn help_flags_go_to_stdout_with_exit_zero() {
        // Short help is concise (examples + a pointer); long help carries
        // the documentation links.
        let short = epher_bin().arg("-h").output().unwrap();
        assert_eq!(short.status.code(), Some(0), "-h exits 0");
        let short_text = String::from_utf8_lossy(&short.stdout);
        assert!(short_text.contains("EXAMPLES:"), "-h leads with examples");
        assert!(short_text.contains("--help"), "-h points at the manual");

        let long = epher_bin().arg("--help").output().unwrap();
        assert_eq!(long.status.code(), Some(0), "--help exits 0");
        let long_text = String::from_utf8_lossy(&long.stdout);
        assert!(
            long_text.contains("epher.org/guide"),
            "--help links the docs"
        );
        assert!(
            long_text.contains("github.com/upyesp/epher/issues"),
            "support path"
        );

        let version = epher_bin().arg("--version").output().unwrap();
        assert_eq!(version.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&version.stdout).starts_with("epher "));
    }

    #[test]
    fn usage_errors_exit_two_on_stderr() {
        // The expression positional and subcommands are mutually
        // exclusive: mixing them is a usage error, not a guess.
        let out = epher_bin().args(["1 + 1", "repl"]).output().unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&out.stderr).contains("cannot be used with"));
        assert!(String::from_utf8_lossy(&out.stdout).is_empty());
    }

    #[test]
    fn help_with_unknown_topic_exits_two_on_stderr() {
        let out = epher_bin().args(["help", "bogus"]).output().unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("unrecognized subcommand 'bogus'"),
            "stderr was: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn help_with_a_topic_prints_that_commands_help_on_stdout() {
        let out = epher_bin().args(["help", "repl"]).output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("interactive session"), "help was: {text}");
        assert!(text.contains("epher repl"), "usage line present: {text}");
    }
}

// --- graph lines and SVG export (ADR-0020) ---

#[test]
fn one_shot_mixes_statements_and_graph_saves() {
    let dir = tempfile::tempdir().unwrap();
    let svg = dir.path().join("plot.svg");
    let input = format!("2 + 2\ngraph sin(x)\ngraph save {}", svg.display());
    let out = epher_bin()
        .env("EPHER_STORE_DIR", dir.path())
        .arg(input)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("4"), "stdout was: {stdout}");
    assert!(stdout.contains("graph: sin(x)"), "stdout was: {stdout}");
    assert!(stdout.contains("saved"), "stdout was: {stdout}");
    let doc = std::fs::read_to_string(&svg).unwrap();
    assert!(doc.starts_with("<svg "), "{doc}");
    assert!(doc.contains("y = sin(x)"), "{doc}");
    assert!(doc.contains("<style>"), "self-contained: {doc}");
}

#[test]
fn piped_scripts_save_svgs_across_lines() {
    let dir = tempfile::tempdir().unwrap();
    let svg = dir.path().join("piped.svg");
    let script = format!("graph x ^ 2 - 1\ngraph save {}\n", svg.display());
    let out = epher_bin()
        .env("EPHER_STORE_DIR", dir.path())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    out.stdin
        .as_ref()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    let out = out.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(svg.exists());
    let doc = std::fs::read_to_string(&svg).unwrap();
    assert!(doc.contains("y = x ^ 2 - 1"), "{doc}");
    // the points of interest carry their labels into the file
    assert!(doc.contains("root"), "{doc}");
}

#[test]
fn repl_graph_save_and_the_empty_diagnostic() {
    let dir = tempfile::tempdir().unwrap();
    let svg = dir.path().join("repl.svg");
    let input = format!(
        "graph save {}\ngraph sin(x)\ngraph save {}\nquit\n",
        svg.display(),
        svg.display()
    );
    let (out, err) = repl_session(dir.path().to_str().unwrap(), &input);
    // saving before anything is plotted is a diagnostic on stderr
    assert!(err.contains("Nothing is plotted"), "stderr was: {err}");
    assert!(!out.contains("Nothing is plotted"), "stdout was: {out}");
    assert!(out.contains("saved"), "stdout was: {out}");
    assert!(svg.exists());
}

#[test]
fn graph3d_save_from_a_one_shot() {
    let dir = tempfile::tempdir().unwrap();
    let svg = dir.path().join("saddle.svg");
    let input = format!("graph3d x ^ 2 - y ^ 2; graph3d save {}", svg.display());
    let out = epher_bin()
        .env("EPHER_STORE_DIR", dir.path())
        .arg(input)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let doc = std::fs::read_to_string(&svg).unwrap();
    assert!(doc.contains("viewBox=\"0 0 640 400\""), "{doc}");
    assert!(doc.contains("transform=\"translate("), "{doc}");
}

// ===== The shared store across frontends (ADR-0010 amendment) =====

#[test]
fn one_shot_uses_the_shared_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // session 1 (REPL): assign x, which sets ans too; both persist.
    repl_output(path, "x = 5\nquit\n");

    // one-shot sees the saved variables; the saved ans is shared too
    let out = epher_bin()
        .arg("ans")
        .env("EPHER_STORE_DIR", path)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");

    let out = epher_bin()
        .arg("x * 2")
        .env("EPHER_STORE_DIR", path)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "10");

    // the one-shot's own evaluation moved ans to 10, and the next
    // one-shot sees that shared value
    let out = epher_bin()
        .arg("ans")
        .env("EPHER_STORE_DIR", path)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "10");

    // the REPL picks up the shared ans too
    let out2 = repl_output(path, "ans + 1\nquit\n");
    assert!(out2.contains("= 11"), "stdout was: {out2}");
}

#[test]
fn one_shot_records_its_command_in_the_shared_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let out = epher_bin()
        .arg("2 + 2")
        .env("EPHER_STORE_DIR", path)
        .output()
        .unwrap();
    assert!(out.status.success());

    // the command joined the store's history setting, alongside the
    // REPL's entries, and the bindings snapshot exists
    let raw = std::fs::read_to_string(dir.path().join("setting/history.json")).unwrap();
    assert!(raw.contains("2 + 2"), "history was: {raw}");
    assert!(dir.path().join("setting/session.json").exists());

    // a later REPL sees the entry in its history (it is display-only,
    // but the entries count as loaded)
    let out2 = repl_output(path, "quit\n");
    assert!(out2.contains("epher>"), "stdout was: {out2}");
}

#[test]
fn solar3d_save_from_a_one_shot() {
    let dir = tempfile::tempdir().unwrap();
    let svg = dir.path().join("solar.svg");
    let input = format!("solar3d jd(2020, 7, 1); solar3d save {}", svg.display());
    let out = epher_bin()
        .env("EPHER_STORE_DIR", dir.path())
        .arg(input)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let doc = std::fs::read_to_string(&svg).unwrap();
    assert!(doc.contains("<circle"), "{doc}");
    assert!(doc.contains("<title>Saturn</title>"), "{doc}");
}
