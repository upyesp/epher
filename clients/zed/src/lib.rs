//! The epher extension for Zed: one job, faithfully.
//!
//! On language-server start the extension resolves the `epher-lsp`
//! binary for this platform from the GitHub release that matches the
//! crate's own version (the locked 0.5.x train, ADR-0066), downloads
//! it into the extension's work directory, and hands the path to Zed.
//! Everything the editor sees after that (diagnostics, inline
//! answers, hover signatures, completion with snippets, semantic
//! tokens) comes from the server itself.

use std::path::Path;

use zed_extension_api::{
    self as zed, current_platform, github_release_by_tag_name, make_file_executable,
    set_language_server_installation_status, Command, DownloadedFileType,
    LanguageServerId, LanguageServerInstallationStatus,
};

/// The repository carrying the `epher-lsp` release assets.
const GITHUB_REPO: &str = "upyesp/epher";

/// The binary name inside a downloaded linux/macos asset (`epher-lsp-<target>.gz`).
const UNIX_BINARY: &str = "epher-lsp";
/// The binary name inside the windows asset (`epher-lsp-windows-x86_64.zip`).
const WINDOWS_BINARY: &str = "epher-lsp.exe";

/// The extension state: which binary path we already prepared this
/// session, so repeated starts skip the download.
struct EpherExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for EpherExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<Command> {
        let path = self.server_binary_path(language_server_id)?;
        Ok(Command {
            command: path,
            args: Vec::new(),
            env: Vec::new(),
        })
    }
}

impl EpherExtension {
    /// Resolve, download if needed, and return the absolute path of
    /// the epher-lsp binary for this platform.
    fn server_binary_path(&mut self, language_server_id: &LanguageServerId) -> zed::Result<String> {
        let (os, arch) = match current_platform() {
            (zed::Os::Linux, zed::Architecture::X8664) => ("linux", "x86_64"),
            (zed::Os::Linux, zed::Architecture::Aarch64) => ("linux", "aarch64"),
            // 32-bit x86 has no builds on any platform.
            (_, zed::Architecture::X86) => {
                return Err("epher does not ship a 32-bit x86 language server".to_string())
            }
            (zed::Os::Mac, zed::Architecture::X8664) => {
                return Err(
                    "epher does not ship an Intel macOS language server yet (ADR-0066 adds it later)"
                        .to_string(),
                )
            }
            (zed::Os::Mac, zed::Architecture::Aarch64) => ("macos", "aarch64"),
            (zed::Os::Windows, zed::Architecture::X8664) => ("windows", "x86_64"),
            (zed::Os::Windows, zed::Architecture::Aarch64) => {
                return Err(
                    "epher does not ship a Windows ARM64 language server yet (ADR-0066 adds it later)"
                        .to_string(),
                )
            }
        };

        let asset_name = if os == "windows" {
            format!("epher-lsp-{os}-{arch}.zip")
        } else {
            format!("epher-lsp-{os}-{arch}.gz")
        };
        let binary_name = if os == "windows" {
            WINDOWS_BINARY
        } else {
            UNIX_BINARY
        };

        // One directory per release version, so an update never mixes
        // binaries; the extension's work directory is the root.
        let version_dir = format!("epher-lsp-{}", env!("CARGO_PKG_VERSION"));
        let binary_path = Path::new(&version_dir).join(binary_name);
        let binary_path = binary_path
            .to_str()
            .ok_or_else(|| "binary path is not valid unicode".to_string())?
            .to_string();

        if self.cached_binary_path.as_deref() == Some(binary_path.as_str())
            && Path::new(&binary_path).exists()
        {
            return Ok(binary_path);
        }

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let tag = format!("v{}", env!("CARGO_PKG_VERSION"));
        let release = github_release_by_tag_name(GITHUB_REPO, &tag).map_err(|error| {
            format!(
                "could not find the epher release {tag} on GitHub: {error}; \
                 the extension version and the release must move together"
            )
        })?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!(
                "the epher release {tag} has no asset named {asset_name}; \
                 this build of the extension does not match the release"
            ))?;

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );

        let file_type = if os == "windows" {
            DownloadedFileType::Zip
        } else {
            DownloadedFileType::Gzip
        };
        zed::download_file(&asset.download_url, &binary_path, file_type)
            .map_err(|error| format!("failed to download {asset_name}: {error}"))?;

        if os != "windows" {
            make_file_executable(&binary_path)
                .map_err(|error| format!("failed to make {binary_path} executable: {error}"))?;
        }

        set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::None,
        );
        self.cached_binary_path = Some(binary_path.clone());

        Ok(binary_path)
    }
}

zed::register_extension!(EpherExtension);
