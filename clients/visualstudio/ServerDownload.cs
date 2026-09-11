// First-use download of the shared language server, the C# twin of
// clients/vscode/src/download.ts (ADR-0066: every extension resolves
// releases/download/v<its own version>/ for a matching server, and
// caches the binary with a server-version marker so the download only
// happens again when the extension updates).
//
// One platform only: Visual Studio runs on Windows, so this client
// always fetches epher-lsp-windows-x86_64.zip (the exe sits at the
// archive root). The linux and macos builds stay irrelevant here, by
// design; if the release lacks the zip the error says so plainly.

using System;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Diagnostics;
using System.Threading.Tasks;

namespace Epher.VisualStudio
{
    internal static class ServerDownload
    {
        private const string Repo = "upyesp/epher";
        private const string Target = "windows-x86_64";
        private const string ExeName = "epher-lsp.exe";

        /// <summary>
        /// Return the cached server, downloading it on first run.
        /// </summary>
        public static async Task<string> EnsureServerAsync()
        {
            var version = ExtensionVersion();

            // %LocalAppData%\epher\bin: the same directory the Windows
            // installer uses for epher's own data, so one well-known
            // place holds everything epher caches on this machine.
            var binDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "epher",
                "bin");
            var exePath = Path.Combine(binDir, ExeName);
            var markerPath = Path.Combine(binDir, "server-version");

            if (File.Exists(exePath)
                && File.Exists(markerPath)
                && File.ReadAllText(markerPath).Trim() == version)
            {
                return exePath;
            }

            var asset = "epher-lsp-" + Target + ".zip";
            var url = "https://github.com/" + Repo + "/releases/download/v" + version + "/" + asset;
            var archive = await DownloadAsync(url);
            Directory.CreateDirectory(binDir);
            ExtractServerExe(archive, exePath);
            File.WriteAllText(markerPath, version);
            return exePath;
        }

        // The extension's own version (stamped into the assembly from
        // the EpherVersion msbuild property, the same number the vsix
        // manifest carries): an extension update re-fetches a matching
        // server, and the versions never drift apart.
        private static string ExtensionVersion()
        {
            return FileVersionInfo.GetVersionInfo(typeof(ServerDownload).Assembly.Location).FileVersion;
        }

        // GET with redirect following: HttpClient's handler follows
        // GitHub's 302s to the actual asset by default
        // (HttpClientHandler.AllowAutoRedirect defaults to true).
        private static async Task<byte[]> DownloadAsync(string url)
        {
            using (var client = new HttpClient())
            {
                client.DefaultRequestHeaders.UserAgent.ParseAdd("epher-visualstudio");
                using (var response = await client.GetAsync(url))
                {
                    if (!response.IsSuccessStatusCode)
                    {
                        var status = (int)response.StatusCode;
                        var hint = response.StatusCode == HttpStatusCode.NotFound
                            ? " (the release has no such asset yet; language-server assets ride the promoted releases)"
                            : "";
                        throw new InvalidOperationException(url + " answered " + status + hint);
                    }
                    return await response.Content.ReadAsByteArrayAsync();
                }
            }
        }

        // Unpack only the server binary: the packaging contract is the
        // exe at the archive's root (build-installers.yml keeps the two
        // in step). Anything else in the zip stays out of the cache.
        private static void ExtractServerExe(byte[] archiveBytes, string exePath)
        {
            using (var stream = new MemoryStream(archiveBytes))
            using (var archive = new ZipArchive(stream, ZipArchiveMode.Read))
            {
                var entry = archive.GetEntry(ExeName)
                    ?? archive.Entries.FirstOrDefault(
                        candidate => string.Equals(candidate.Name, ExeName, StringComparison.OrdinalIgnoreCase));
                if (entry == null)
                {
                    throw new InvalidOperationException(
                        "epher-lsp-" + Target + ".zip carries no " + ExeName + " at the archive root");
                }
                entry.ExtractToFile(exePath, overwrite: true);
            }
        }
    }
}
