# The store manifest hides the console application under the approved headless waiver

Date: 2026-11-21.

Status: Accepted. Supersedes ADR-0071 and ADR-0074; repeals ADR-0074's
2026-10-11 amendment of ADR-0011.

## Context

ADR-0071 shipped two visible Store applications because the first
package - the original shape, with the console build hidden under
`AppListEntry="none"` - was rejected as a headless app, and the store
grants exceptions only through the HeadlessAppBypass waiver, which the
listing did not have. The waiver request stayed open with Microsoft
support (case TrackingID#2610020050003316). Their first reply asked why
the current package shows no HeadlessAppBypass error - the answer being
ADR-0071 itself: nothing is hidden any more - and, once that was
explained, what it takes for the waiver to be approved. On 2026-11-21
Microsoft confirmed the request approved: HeadlessAppBypass is enabled
for product epher (Store ID 9NWXQ035T8TB), with the instruction to
proceed with the intended configuration and report back whether it
works.

## Decision

The manifest returns to the original shape the waiver was requested
for (research doc 1.3). `EpherGuiApp` (`epher-gui.exe`) stays the only
Start tile and keeps the `.epher` file type association. `EpherApp`
(`epher.exe`) keeps the `epher.exe` console execution alias - still the
only way an MSIX can put `epher` on PATH - and its VisualElements carry
`AppListEntry="none"` again, so the Start menu shows one entry. The
ADR-0074 bare-run banner is removed with the shape it served: a bare
`epher` on the Windows console build goes back to ADR-0011's rule and
opens the window through the sibling hand-off, on the Store install and
the NSIS install alike.

## Consequences

- The Start menu shows one entry, `epher`; the CLI stays reachable as
  `epher` in every terminal through the execution alias.
- Certification is the live test of the waiver: the next Store update
  submission must pass with the hidden entry, and a pass is reported
  back to Microsoft on the same case, as they asked.
- The ADR-0074 banner, keypress wait, and console-mode code are gone,
  and so is the `windows-sys` dependency on the tauri-app crate;
  `windows-sys` itself stays in the lockfile through the Tauri stack,
  so the flatpak vendor manifest is unchanged.
- If certification ever fails again as headless, the written waiver
  approval on case TrackingID#2610020050003316 is the record to point
  at; the ADR-0071 two-visible shape remains the documented fallback.
