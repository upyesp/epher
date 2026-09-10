import * as fs from "fs";
import * as path from "path";
import * as https from "https";
import * as zlib from "zlib";
import { execFile } from "child_process";
import * as vscode from "vscode";

const REPO = "upyesp/epher";

// The platforms the pilot ships (ADR-0066: today's four first). The
// key is Node's platform-arch spelling, the value the release asset's.
const TARGETS: Record<string, string> = {
  "linux-x64": "linux-x86_64",
  "linux-arm64": "linux-aarch64",
  "darwin-arm64": "macos-aarch64",
  "win32-x64": "windows-x86_64",
};

/// Return the cached server, downloading it on first run. The URL is
/// pinned to the extension's own version
/// (`releases/download/v<version>/epher-lsp-<target>`), so an
/// extension update re-fetches a matching server (ADR-0066).
export async function ensureServer(
  context: vscode.ExtensionContext,
  channel: vscode.OutputChannel,
): Promise<string> {
  const platform = `${process.platform}-${process.arch}`;
  const target = TARGETS[platform];
  if (!target) {
    throw new Error(
      `no epher-lsp build for ${platform}; the pilot ships for ${Object.values(TARGETS).join(", ")}`,
    );
  }
  const version: string = context.extension.packageJSON.version;
  const binDir = path.join(context.globalStorageUri.fsPath, "bin");
  const exe = path.join(
    binDir,
    process.platform === "win32" ? "epher-lsp.exe" : "epher-lsp",
  );
  const marker = path.join(binDir, "server-version");
  if (
    fs.existsSync(exe) &&
    fs.existsSync(marker) &&
    fs.readFileSync(marker, "utf8").trim() === version
  ) {
    channel.appendLine(`epher-lsp ${version} already cached: ${exe}`);
    return exe;
  }

  const asset = `epher-lsp-${target}.${process.platform === "win32" ? "zip" : "gz"}`;
  const url = `https://github.com/${REPO}/releases/download/v${version}/${asset}`;
  channel.appendLine(`downloading ${url}`);
  const archive = await download(url);
  fs.mkdirSync(binDir, { recursive: true });
  if (asset.endsWith(".gz")) {
    fs.writeFileSync(exe, zlib.gunzipSync(archive));
  } else {
    // Windows rides a zip; Expand-Archive unpacks the binary at the
    // archive's root.
    const zipPath = path.join(binDir, asset);
    fs.writeFileSync(zipPath, archive);
    await expandArchive(zipPath, binDir);
    fs.rmSync(zipPath);
  }
  if (process.platform !== "win32") {
    fs.chmodSync(exe, 0o755);
  }
  fs.writeFileSync(marker, version);
  channel.appendLine(`epher-lsp ${version} ready: ${exe}`);
  return exe;
}

/// GET with redirect following; GitHub's release assets answer 302.
function download(url: string, redirects = 0): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    if (redirects > 5) {
      reject(new Error("too many redirects"));
      return;
    }
    const req = https.get(
      url,
      { headers: { "User-Agent": "epher-vscode" } },
      (res) => {
        const status = res.statusCode ?? 0;
        if (status >= 300 && status < 400 && res.headers.location) {
          res.resume();
          download(new URL(res.headers.location, url).toString(), redirects + 1).then(
            resolve,
            reject,
          );
          return;
        }
        if (status !== 200) {
          res.resume();
          const hint =
            status === 404
              ? " (the release has no such asset yet; language-server assets ride the promoted releases)"
              : "";
          reject(new Error(`${url} answered ${status}${hint}`));
          return;
        }
        const chunks: Buffer[] = [];
        res.on("data", (chunk: Buffer) => chunks.push(chunk));
        res.on("end", () => resolve(Buffer.concat(chunks)));
        res.on("error", reject);
      },
    );
    req.on("error", reject);
  });
}

/// Unpack a zip on Windows without adding a dependency: PowerShell is
/// on every Windows the pilot targets.
function expandArchive(zip: string, dir: string): Promise<void> {
  return new Promise((resolve, reject) => {
    execFile(
      "powershell.exe",
      [
        "-NoProfile",
        "-Command",
        `Expand-Archive -LiteralPath '${zip}' -DestinationPath '${dir}' -Force`,
      ],
      (err) => (err ? reject(err) : resolve()),
    );
  });
}
