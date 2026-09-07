//! Terminal output policy (ADR-0013, clig.dev): results on stdout,
//! diagnostics on stderr, and color only where it helps — anstream
//! detects the terminal and honors NO_COLOR, TERM=dumb, and
//! CLICOLOR_FORCE per stream automatically.

/// Print an error diagnostic to stderr: red when stderr is a terminal
/// the user hasn't asked to keep plain, plain text otherwise.
pub fn error(message: &str) {
    let style = anstyle::Style::new()
        .bold()
        .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)));
    anstream::eprintln!("{style}{message}{style:#}");
}
