//! The macOS "install the `epher` command" seam (ADR-0011): pure helpers
//! that decide where the symlink lives and how the osascript fallback is
//! built. Side effects live in thin Tauri commands over these.

use std::path::{Path, PathBuf};

/// The install plan: (link, target). `Some` only when `exe` is a macOS app
/// bundle executable (`…/epher.app/Contents/MacOS/epher`) — dev runs and
/// other platforms get `None` and the UI never offers the action.
///
/// `/usr/local/bin` is on every default macOS PATH and survives reboots,
/// unlike shell-rc aliases.
pub fn cli_symlink_plan(exe: &Path) -> Option<(PathBuf, PathBuf)> {
    let comps: Vec<_> = exe.components().collect();
    if comps.len() < 4 {
        return None;
    }
    let n = comps.len();
    let as_str = |c: &std::path::Component| c.as_os_str().to_string_lossy().into_owned();
    let is_bundle = as_str(&comps[n - 1]) == "epher"
        && as_str(&comps[n - 2]) == "MacOS"
        && as_str(&comps[n - 3]) == "Contents"
        && as_str(&comps[n - 4]).ends_with(".app");
    if !is_bundle {
        return None;
    }
    Some((PathBuf::from("/usr/local/bin/epher"), exe.to_path_buf()))
}

/// POSIX single-quote a string for embedding in a shell command.
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Escape a shell command for embedding inside an AppleScript string
/// literal (double quotes): `\` and `"` must be escaped.
pub fn applescript_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The administrator-privilege fallback when `/usr/local/bin` is not
/// writable directly: an osascript line that shows the native password
/// prompt and runs the command.
pub fn osascript_line(shell_command: &str) -> String {
    format!(
        "do shell script \"{}\" with administrator privileges",
        applescript_escape(shell_command)
    )
}

/// The copy-paste manual fallback shown when the user cancels the prompt.
pub fn manual_instructions(target: &Path, link: &Path) -> String {
    format!(
        "run: sudo ln -sf -- {} {}",
        shell_quote(&target.to_string_lossy()),
        shell_quote(&link.to_string_lossy())
    )
}

/// The side-effectful install, shared by the Tauri command. Returns a
/// Fluent key on success (`install-cli-ok` / `install-cli-already`); a
/// human-readable fallback string on failure.
pub fn install() -> Result<String, String> {
    #[cfg(not(target_os = "macos"))]
    {
        Err("the epher command can only be installed on macOS".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        let exe = std::env::current_exe().map_err(|e| format!("could not locate the app: {e}"))?;
        let Some((link, target)) = cli_symlink_plan(&exe) else {
            return Err(format!("not running from an app bundle: {}", exe.display()));
        };
        // Already installed (our symlink, pointing here)?
        if std::fs::read_link(&link)
            .map(|existing| existing == target)
            .unwrap_or(false)
        {
            return Ok("install-cli-already".to_string());
        }
        // Stale or absent link: try directly first (many Macs have
        // user-writable /usr/local/bin; no password prompt needed).
        let _ = std::fs::remove_file(&link);
        if std::os::unix::fs::symlink(&target, &link).is_ok() {
            return Ok("install-cli-ok".to_string());
        }
        // Permission needed: native administrator prompt via osascript.
        let sh = format!(
            "ln -sf -- {} {}",
            shell_quote(&target.to_string_lossy()),
            shell_quote(&link.to_string_lossy())
        );
        let out = std::process::Command::new("osascript")
            .arg("-e")
            .arg(osascript_line(&sh))
            .output()
            .map_err(|e| format!("could not run osascript: {e}"))?;
        if out.status.success() {
            Ok("install-cli-ok".to_string())
        } else {
            Err(manual_instructions(&target, &link))
        }
    }
}
