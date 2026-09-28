# Sizes the Visual Studio window to exactly 1244x900 at the top-left
# of the screen, to match the frame size of the other listings' shots.
# Run AFTER Visual Studio has opened demo.epher.

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win {
  [DllImport("user32.dll")]
  public static extern bool MoveWindow(IntPtr hWnd, int X, int Y, int nWidth, int nHeight, bool bRepaint);
}
"@

$p = Get-Process devenv -ErrorAction SilentlyContinue |
     Where-Object { $_.MainWindowHandle -ne 0 } |
     Select-Object -First 1

if (-not $p) {
  Write-Error "no Visual Studio window found (is devenv running?)"
  exit 1
}

[Win]::MoveWindow($p.MainWindowHandle, 0, 0, 1244, 900, $true) | Out-Null
Write-Output "devenv moved to 1244x900 at (0,0)"
