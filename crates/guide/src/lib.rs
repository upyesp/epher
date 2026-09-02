//! The epher user guide, shared by the web app and the TUI (ADR-0018).
//!
//! The single source of truth is `site/guide/<locale>.md`, the same
//! files the website build (`scripts/build-guide.mjs`) turns into the
//! guide pages. No binary carries the guide: the build script copies
//! the markdown into the web app's static files (fetched on demand),
//! and the installers put it on disk (the TUI reads it when the user
//! opens the guide) — ADR-0053. Each frontend renders the markdown
//! with the renderer that fits its medium: [`render_html`] for the
//! web/desktop overlay (every `` ```epher `` / `` ```sh `` fence becomes
//! a clickable example button), [`render_text`] for the TUI pager. The
//! markdown feature set is bounded by the guide itself: ATX headings
//! (1–4), fenced code blocks (`epher`, `sh`, `text`), flat lists, pipe
//! tables, blockquotes, paragraphs, and the inline marks `` `code` ``,
//! `**bold**`, `*italic*`. No links, images, or raw HTML; so both
//! renderers stay small and every byte of text is escaped.

/// The locales the guide ships in, matching site/guide/*.md.
pub const LOCALES: [&str; 8] = ["ar", "de", "en", "es", "fr", "hi", "pt", "zh-CN"];

/// The guide file name for a locale: `<locale>.md` for the shipped
/// locales, `en.md` otherwise (the website falls back the same way).
pub fn file_name(locale: &str) -> String {
    if LOCALES.contains(&locale) {
        format!("{locale}.md")
    } else {
        "en.md".to_string()
    }
}

/// Why the guide could not be read: the installed files were not found.
/// The message lists every directory that was tried so the user can fix
/// the install (or set `EPHER_GUIDE_DIR`).
#[derive(Debug)]
pub struct GuideUnavailable {
    /// The directories that were tried, in order.
    pub tried: Vec<String>,
}

impl std::fmt::Display for GuideUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the guide files were not found; looked in:")?;
        for dir in &self.tried {
            write!(f, "\n  {dir}")?;
        }
        Ok(())
    }
}

impl std::error::Error for GuideUnavailable {}

/// Read the guide for a locale from the installed files. Native
/// frontends only; the web app fetches the same files over HTTP.
/// Search order:
///
/// 1. `$EPHER_GUIDE_DIR` (explicit override, one file per locale)
/// 2. next to the executable: `guide/` and `resources/guide/`
///    (the NSIS install layout)
/// 3. bundle-relative: `../Resources/guide/` (the macOS app bundle's
///    Contents/Resources) and `../lib/<name>/resources/guide/`
///    (the Linux resource dir next to /usr/bin)
/// 4. the system data dirs `/usr/local/share/epher/guide/` and
///    `/usr/share/epher/guide/` (the deb/rpm file maps)
/// 5. the user's data dir (`$XDG_DATA_HOME` or `~/.local/share`) —
///    `epher/guide/` there, for installs without a package
#[cfg(not(target_arch = "wasm32"))]
pub fn load(locale: &str) -> Result<String, GuideUnavailable> {
    let file = file_name(locale);
    let mut tried = Vec::new();

    let push = |dir: std::path::PathBuf, tried: &mut Vec<String>| {
        let joined = dir.join(&file);
        let shown = dir.display().to_string();
        if !tried.contains(&shown) {
            tried.push(shown);
        }
        std::fs::read_to_string(&joined).ok()
    };

    if let Some(dir) = std::env::var_os("EPHER_GUIDE_DIR") {
        if let Some(md) = push(std::path::PathBuf::from(dir), &mut tried) {
            return Ok(md);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            for rel in ["guide", "resources/guide", "../Resources/guide"] {
                if let Some(md) = push(parent.join(rel), &mut tried) {
                    return Ok(md);
                }
            }
            // The Linux bundle layout: /usr/bin/epher with resources in
            // /usr/lib/<productName>/resources (Tauri's resource dir),
            // reached relative so AppImage works the same way.
            if let Some(bin) = parent.parent() {
                if let Some(md) = push(bin.join("lib/epher/resources/guide"), &mut tried) {
                    return Ok(md);
                }
            }
        }
    }
    for dir in ["/usr/local/share/epher/guide", "/usr/share/epher/guide"] {
        if let Some(md) = push(std::path::PathBuf::from(dir), &mut tried) {
            return Ok(md);
        }
    }
    if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
        if let Some(md) = push(std::path::PathBuf::from(data).join("epher/guide"), &mut tried) {
            return Ok(md);
        }
    } else if let Some(home) = std::env::var_os("HOME") {
        if let Some(md) = push(
            std::path::PathBuf::from(home).join(".local/share/epher/guide"),
            &mut tried,
        ) {
            return Ok(md);
        }
    }
    Err(GuideUnavailable { tried })
}

/// Escape text for inclusion in generated HTML.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Inline markdown → HTML. Escapes first, then applies the three inline
/// marks the guide uses. `` `code` `` spans win over everything else, and
/// `**bold**` is applied before `*italic*`.
fn inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, seg) in s.split('`').enumerate() {
        if i % 2 == 1 {
            // A code span: no further marks inside.
            out.push_str("<code>");
            out.push_str(&escape_html(seg));
            out.push_str("</code>");
            continue;
        }
        for (j, part) in seg.split("**").enumerate() {
            if j % 2 == 1 {
                out.push_str("<strong>");
            }
            for (k, sub) in part.split('*').enumerate() {
                if k % 2 == 1 {
                    out.push_str("<em>");
                }
                out.push_str(&escape_html(sub));
                if k % 2 == 1 {
                    out.push_str("</em>");
                }
            }
            if j % 2 == 1 {
                out.push_str("</strong>");
            }
        }
    }
    out
}

/// Strip the inline marks for plain-text output.
fn strip_inline(s: &str) -> String {
    s.replace("**", "").replace('*', "").replace('`', "")
}

/// `#`–`####` → (level, rest); nothing else.
fn heading(line: &str) -> Option<(u8, &str)> {
    for (lvl, mark) in [(4, "#### "), (3, "### "), (2, "## "), (1, "# ")] {
        if let Some(rest) = line.strip_prefix(mark) {
            return Some((lvl, rest));
        }
    }
    None
}

fn is_table_separator(line: &str) -> bool {
    line.starts_with('|') && line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn table_cells(line: &str) -> Vec<&str> {
    let trimmed = line.trim().trim_start_matches('|').trim_end_matches('|');
    trimmed.split('|').map(str::trim).collect()
}

fn is_ol_line(l: &str) -> bool {
    let digits = l.chars().take_while(|c| c.is_ascii_digit()).count();
    digits > 0 && l[digits..].starts_with(". ")
}

/// The guide's top-level chapters: the `## ` heading titles, in order,
/// with inline marks stripped. The in-app table of contents (ADR-0018
/// amendment) is built from this list in every frontend, so the ToC is
/// automatically localized with the guide.
pub fn chapters(md: &str) -> Vec<String> {
    md.lines()
        .filter_map(|l| l.strip_prefix("## "))
        .map(strip_inline)
        .collect()
}

/// Render the guide markdown as HTML for the web/desktop overlay.
///
/// `` ```epher `` and `` ```sh `` fences become clickable example blocks:
/// a `<button class="guide-example-btn">` whose `data-code` attribute
/// carries the exact text to insert into the entry field. `` ```text ``
/// fences are plain output boxes. `## ` headings carry `id="guide-ch-N"`
/// anchors, and the document opens with a `<nav class="guide-toc">`
/// whose buttons (`data-jump="N"`) scroll to them. The in-app table of
/// contents (ADR-0018 amendment). Everything else maps to the obvious
/// element; all text is escaped.
pub fn render_html(md: &str, toc_label: &str) -> String {
    let lines: Vec<&str> = md.lines().collect();
    let mut out = String::with_capacity(md.len() * 2);
    let mut chapter = 0usize;
    let mut toc = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            i += 1;
            continue;
        }
        if let Some(lang) = line.strip_prefix("```") {
            let lang = lang.trim();
            i += 1;
            let mut code = String::new();
            while i < lines.len() && !lines[i].starts_with("```") {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(lines[i]);
                i += 1;
            }
            i += 1; // closing fence
            match lang {
                "epher" | "sh" => {
                    out.push_str("<div class=\"guide-example\"><button type=\"button\" class=\"guide-example-btn\" data-code=\"");
                    out.push_str(&escape_html(&code));
                    out.push_str("\">");
                    out.push_str(&escape_html(&code));
                    out.push_str("</button></div>");
                }
                _ => {
                    out.push_str("<pre class=\"guide-out\"><code>");
                    out.push_str(&escape_html(&code));
                    out.push_str("</code></pre>");
                }
            }
            continue;
        }
        if let Some((lvl, rest)) = heading(line) {
            let tag = match lvl {
                2 => "h2",
                3 => "h3",
                4 => "h4",
                _ => "h1",
            };
            if lvl == 2 {
                let n = chapter;
                out.push_str(&format!("<h2 id=\"guide-ch-{n}\">{}</h2>", inline(rest)));
                toc.push_str(&format!(
                    "<li><button type=\"button\" class=\"guide-toc-btn\" data-jump=\"{n}\">{}</button></li>",
                    inline(rest)
                ));
                chapter += 1;
            } else {
                out.push_str(&format!("<{tag}>{}</{tag}>", inline(rest)));
            }
            i += 1;
            continue;
        }
        if line.starts_with('|') && i + 1 < lines.len() && is_table_separator(lines[i + 1]) {
            let header = table_cells(line);
            i += 2;
            let mut rows = Vec::new();
            while i < lines.len() && lines[i].starts_with('|') {
                rows.push(table_cells(lines[i]));
                i += 1;
            }
            out.push_str("<table><thead><tr>");
            for c in &header {
                out.push_str(&format!("<th>{}</th>", inline(c)));
            }
            out.push_str("</tr></thead><tbody>");
            for row in &rows {
                out.push_str("<tr>");
                for n in 0..header.len() {
                    let cell = row.get(n).copied().unwrap_or("");
                    out.push_str(&format!("<td>{}</td>", inline(cell)));
                }
                out.push_str("</tr>");
            }
            out.push_str("</tbody></table>");
            continue;
        }
        if line == ">" || line.starts_with("> ") {
            let mut parts = Vec::new();
            while i < lines.len() && (lines[i] == ">" || lines[i].starts_with("> ")) {
                let rest = lines[i].strip_prefix("> ").unwrap_or("");
                if !rest.is_empty() {
                    parts.push(rest);
                }
                i += 1;
            }
            out.push_str("<blockquote><p>");
            out.push_str(&inline(&parts.join(" ")));
            out.push_str("</p></blockquote>");
            continue;
        }
        if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            out.push_str("<ul>");
            out.push_str(&format!("<li>{}</li>", inline(item)));
            i += 1;
            while i < lines.len() {
                match lines[i]
                    .strip_prefix("- ")
                    .or_else(|| lines[i].strip_prefix("* "))
                {
                    Some(item) => {
                        out.push_str(&format!("<li>{}</li>", inline(item)));
                        i += 1;
                    }
                    None => break,
                }
            }
            out.push_str("</ul>");
            continue;
        }
        if is_ol_line(line) {
            out.push_str("<ol>");
            while i < lines.len() && is_ol_line(lines[i]) {
                let dot = lines[i].find('.').unwrap();
                out.push_str(&format!("<li>{}</li>", inline(&lines[i][dot + 2..])));
                i += 1;
            }
            out.push_str("</ol>");
            continue;
        }
        let mut parts = vec![line];
        i += 1;
        while i < lines.len()
            && !lines[i].trim().is_empty()
            && !lines[i].starts_with("```")
            && heading(lines[i]).is_none()
            && !lines[i].starts_with('|')
            && lines[i] != ">"
            && !lines[i].starts_with("> ")
            && !lines[i].starts_with("- ")
            && !lines[i].starts_with("* ")
            && !is_ol_line(lines[i])
        {
            parts.push(lines[i]);
            i += 1;
        }
        out.push_str("<p>");
        out.push_str(&inline(&parts.join(" ")));
        out.push_str("</p>");
    }
    format!(
        "<nav class=\"guide-toc\" aria-label=\"{}\"><h3 class=\"guide-toc-heading\">{}</h3><ol>{}</ol></nav>{}",
        escape_html(toc_label),
        escape_html(toc_label),
        toc,
        out
    )
}

/// One line of the plain-text rendering for the TUI pager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TLine {
    /// Chapter/section heading; the u8 is the heading level (1–4).
    Heading(u8, String),
    /// Body text, list items, and table rows.
    Text(String),
    /// Fenced code (examples and transcripts).
    Code(String),
    /// Blockquote paragraphs.
    Quote(String),
    /// An empty line.
    Blank,
}

/// Render the guide markdown as plain text lines for the TUI pager. The
/// same parser as [`render_html`], minus the inline styling.
pub fn render_text(md: &str) -> Vec<TLine> {
    let lines: Vec<&str> = md.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            out.push(TLine::Blank);
            i += 1;
            continue;
        }
        if let Some(lang) = line.strip_prefix("```") {
            i += 1;
            let mut code = Vec::new();
            while i < lines.len() && !lines[i].starts_with("```") {
                code.push(lines[i].to_string());
                i += 1;
            }
            i += 1;
            let _ = lang;
            out.push(TLine::Blank);
            for c in code {
                out.push(TLine::Code(c));
            }
            out.push(TLine::Blank);
            continue;
        }
        if let Some((lvl, rest)) = heading(line) {
            out.push(TLine::Heading(lvl, strip_inline(rest)));
            i += 1;
            continue;
        }
        if line.starts_with('|') && i + 1 < lines.len() && is_table_separator(lines[i + 1]) {
            let header = table_cells(line);
            i += 2;
            out.push(TLine::Text(header.join(" | ")));
            while i < lines.len() && lines[i].starts_with('|') {
                out.push(TLine::Text(table_cells(lines[i]).join(" | ")));
                i += 1;
            }
            continue;
        }
        if line == ">" || line.starts_with("> ") {
            let mut parts = Vec::new();
            while i < lines.len() && (lines[i] == ">" || lines[i].starts_with("> ")) {
                let rest = lines[i].strip_prefix("> ").unwrap_or("");
                if !rest.is_empty() {
                    parts.push(rest);
                }
                i += 1;
            }
            out.push(TLine::Quote(strip_inline(&parts.join(" "))));
            continue;
        }
        if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            out.push(TLine::Text(format!("• {}", strip_inline(item))));
            i += 1;
            while i < lines.len() {
                match lines[i]
                    .strip_prefix("- ")
                    .or_else(|| lines[i].strip_prefix("* "))
                {
                    Some(item) => {
                        out.push(TLine::Text(format!("• {}", strip_inline(item))));
                        i += 1;
                    }
                    None => break,
                }
            }
            continue;
        }
        if is_ol_line(line) {
            let mut n = 1;
            while i < lines.len() && is_ol_line(lines[i]) {
                let dot = lines[i].find('.').unwrap();
                out.push(TLine::Text(format!(
                    "{n}. {}",
                    strip_inline(&lines[i][dot + 2..])
                )));
                n += 1;
                i += 1;
            }
            continue;
        }
        let mut parts = vec![line];
        i += 1;
        while i < lines.len()
            && !lines[i].trim().is_empty()
            && !lines[i].starts_with("```")
            && heading(lines[i]).is_none()
            && !lines[i].starts_with('|')
            && lines[i] != ">"
            && !lines[i].starts_with("> ")
            && !lines[i].starts_with("- ")
            && !lines[i].starts_with("* ")
            && !is_ol_line(lines[i])
        {
            parts.push(lines[i]);
            i += 1;
        }
        out.push(TLine::Text(strip_inline(&parts.join(" "))));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The repo's site/guide: the single source of truth the build
    /// script copies for the web app (ADR-0053).
    fn site_guide() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../site/guide")
    }

    #[test]
    fn file_name_maps_locales_and_falls_back_to_english() {
        assert_eq!(file_name("en"), "en.md");
        assert_eq!(file_name("de"), "de.md");
        assert_eq!(file_name("zh-CN"), "zh-CN.md");
        assert_eq!(file_name("xx"), "en.md");
    }

    #[test]
    fn load_reads_the_installed_guide_and_reports_misses() {
        // The happy path through the env override (how a source checkout
        // or a custom install points at the files).
        std::env::set_var("EPHER_GUIDE_DIR", site_guide());
        let en = load("en").expect("guide en loads from site/guide");
        assert!(en.contains("epher user guide"));
        // Unknown locale falls back to the English file.
        let xx = load("xx").expect("guide xx falls back to en");
        assert_eq!(xx, en);
        std::env::remove_var("EPHER_GUIDE_DIR");
        // No installed files: the error lists where it looked.
        let dir = std::env::temp_dir().join(format!("epher-guide-miss-{}", std::process::id()));
        std::env::set_var("EPHER_GUIDE_DIR", &dir);
        let err = load("en").expect_err("empty dir has no guide");
        assert!(err.tried.iter().any(|d| d.contains(&dir.display().to_string())));
        std::env::remove_var("EPHER_GUIDE_DIR");
    }

    #[test]
    fn html_headings_lists_and_paragraphs() {
        let md = "# Title\n\nIntro line.\nwrapped line.\n\n- one\n- two\n\n1. first\n2. second\n";
        let h = render_html(md, "Contents");
        assert!(h.contains("<h1>Title</h1>"), "{h}");
        assert!(h.contains("<p>Intro line. wrapped line.</p>"), "{h}");
        assert!(h.contains("<ul><li>one</li><li>two</li></ul>"), "{h}");
        assert!(h.contains("<ol><li>first</li><li>second</li></ol>"), "{h}");
    }

    #[test]
    fn html_epher_fences_become_clickable_examples_and_text_is_escaped() {
        let md = "```epher\n2 + 3 * 4\n```\n\n```text\n14\n```\n";
        let h = render_html(md, "Contents");
        assert!(
            h.contains("class=\"guide-example-btn\" data-code=\"2 + 3 * 4\">2 + 3 * 4"),
            "{h}"
        );
        assert!(
            h.contains("<pre class=\"guide-out\"><code>14</code></pre>"),
            "{h}"
        );
        // angle brackets and quotes inside code are escaped in the attribute
        let md2 = "```epher\nx > 1 and \"a\"\n```\n";
        let h2 = render_html(md2, "Contents");
        assert!(
            h2.contains("data-code=\"x &gt; 1 and &quot;a&quot;\""),
            "{h2}"
        );
    }

    #[test]
    fn html_tables_blockquotes_and_inline() {
        let md = "| a | b |\n|---|---|\n| 1 | 2 |\n\n> note **bold** and `code`.\n\nPara with **b** and *i* and `c`.\n";
        let h = render_html(md, "Contents");
        assert!(h.contains("<table><thead><tr><th>a</th><th>b</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table>"), "{h}");
        assert!(
            h.contains(
                "<blockquote><p>note <strong>bold</strong> and <code>code</code>.</p></blockquote>"
            ),
            "{h}"
        );
        assert!(
            h.contains("Para with <strong>b</strong> and <em>i</em> and <code>c</code>."),
            "{h}"
        );
    }

    #[test]
    fn html_h4_headings_survive() {
        let h = render_html("#### Small\n", "Contents");
        assert!(h.contains("<h4>Small</h4>"), "{h}");
    }

    #[test]
    fn full_guides_in_every_locale_render_without_panic() {
        for l in ["ar", "de", "en", "es", "fr", "hi", "pt", "zh-CN"] {
            let md = std::fs::read_to_string(site_guide().join(format!("{l}.md")))
                .unwrap_or_else(|e| panic!("guide {l}: {e}"));
            assert!(md.len() > 5000, "guide {l} suspiciously short");
            let html = render_html(&md, "Contents");
            assert!(html.contains("<h1>"), "guide {l}: no h1 in HTML");
            assert!(html.contains("guide-example-btn"), "guide {l}: no examples");
            let text = render_text(&md);
            assert!(!text.is_empty());
            assert!(text.iter().any(|t| matches!(t, TLine::Heading(1, _))));
        }
    }

    #[test]
    fn text_mode_strips_marks_and_keeps_structure() {
        let md = "## Chap\n\nPara `code` **bold**.\n\n```epher\n1+1\n```\n\n> note\n";
        let t = render_text(md);
        assert_eq!(t[0], TLine::Heading(2, "Chap".into()));
        assert!(t.contains(&TLine::Text("Para code bold.".into())));
        assert!(t.contains(&TLine::Code("1+1".into())));
        assert!(t.contains(&TLine::Quote("note".into())));
    }
}
