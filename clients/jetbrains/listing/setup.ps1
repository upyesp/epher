#Requires -Version 7

# Prepares a Windows machine for the JetBrains Marketplace captures:
# installs IntelliJ IDEA Ultimate 2024.2.4 (the exact IDE line the
# plugin builds against; free 30-day trial, a JetBrains Account login
# is requested at first start - the same account used for the
# marketplace portal), installs the epher plugin from disk, and creates
# the demo project with demo.epher.
#
#   pwsh -File setup.ps1 [-PluginZip epher-0.5.58.zip]
#
# Idempotent: safe to rerun. Downloaded artifacts land in %TEMP%.

param(
    [string]$PluginZip = "",
    [string]$IdeVersion = "2024.2.4",
    [string]$IdeDir = "$env:LOCALAPPDATA\Programs\ideaIU-captures",
    [string]$ProjectDir = "$env:USERPROFILE\Desktop\epher-captures"
)

$ErrorActionPreference = "Stop"

function Fetch($url, $out) {
    if (Test-Path $out) { Write-Host "have $(Split-Path -Leaf $out)"; return }
    Write-Host "downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $out
}

# --- 1. the IDE ---------------------------------------------------------
$installer = Join-Path $env:TEMP "ideaIU-$IdeVersion.exe"
$ver = $IdeVersion.Replace('.', '')
Fetch "https://download.jetbrains.com/idea/ideaIU-$IdeVersion.exe" $installer

if (-not (Test-Path (Join-Path $IdeDir "bin\idea64.exe"))) {
    Write-Host "installing IntelliJ IDEA $IdeVersion into $IdeDir (silent)"
    $p = Start-Process -FilePath $installer -ArgumentList @(
        "/S", "/D=$IdeDir"
    ) -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw "IDE installer exited $($p.ExitCode)" }
} else {
    Write-Host "IDE already installed"
}

# --- 2. the plugin ------------------------------------------------------
if (-not $PluginZip) {
    # newest epher-jetbrains*.zip on the Desktop, else the latest release
    $candidates = Get-ChildItem "$env:USERPROFILE\Desktop\epher-jetbrains*.zip" -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending
    if ($candidates) {
        $PluginZip = $candidates[0].FullName
    } else {
        $PluginZip = Join-Path $env:TEMP "epher-jetbrains.zip"
        Fetch "https://github.com/upyesp/epher/releases/latest/download/epher-jetbrains.zip" $PluginZip
    }
}
if (-not (Test-Path $PluginZip)) { throw "plugin zip not found: $PluginZip" }
Write-Host "plugin zip: $PluginZip"

# --- 3. the demo project ------------------------------------------------
New-Item -ItemType Directory -Force -Path $ProjectDir | Out-Null
$demo = Join-Path $ProjectDir "demo.epher"
@'
// the earth, measured
const radius = 6371 km
const diameter = 2 * radius
circumference = pi * diameter

// the same numbers in miles
circumference in mile

// a right triangle
const legs = 3 m
const hypotenuse = 5 m
height = sqrt(hypotenuse^2 - legs^2)

// the discriminant
def disc(a, b, c) = b^2 - 4*a*c
disc(1, -5, 6)

graph sin(x)
'@ | Set-Content -Encoding utf8NoBOM $demo
Set-Content -Encoding utf8NoBOM (Join-Path $ProjectDir "notes.txt") "epher demo project"

# --- 4. launch ----------------------------------------------------------
Write-Host ""
Write-Host "Next steps (manual):"
Write-Host "  1. Start '$IdeDir\bin\idea64.exe' and open the folder '$ProjectDir'."
Write-Host "  2. Sign in (or start the trial) when asked; EAP/trial is free."
Write-Host "  3. Plugins, gear menu, Install Plugin from Disk -> $PluginZip, restart."
Write-Host "  4. Reopen '$demo': the inline answers appear after the one-time"
Write-Host "     language-server download."
Write-Host "  5. Follow CAPTURE-GUIDE.md for the shots."
Write-Host ""
Write-Host "To install the plugin now, start the IDE with:"
Write-Host "  & '$IdeDir\bin\idea64.exe'"
