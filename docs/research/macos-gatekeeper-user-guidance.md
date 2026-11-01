# macOS Gatekeeper guidance in the wild: what cask vendors, Homebrew and Apple tell users about the "Apple could not verify" dialog — and what that means for our brew note

Research of 2026-11-01 (branch `staging`, uncommitted), while the site carries a
first-launch hint under the macOS card ("Choose Done, run `xattr -rd …`, export
`HOMEBREW_CASK_OPTS=--no-quarantine`"). Three questions: what do the vendors
behind well-known Homebrew casks actually tell their users about Gatekeeper,
what do Homebrew and Apple themselves promise, and how does our note compare?
The headline: **vendor guidance has all but vanished** — of 20 surveyed casks,
only two publish anything Gatekeeper-shaped, and not one recommends a bypass;
**Apple now documents exactly one remedy** (System Settings → Privacy & Security
→ "Open Anyway", the Control-click → Open bypass is gone from its page);
**Homebrew reversed its old "at your own risk" stance in 5.0.0** — Gatekeeper
checks are now a cask requirement, `--no-quarantine`/`--quarantine` were
deprecated (Nov 2025) and the leftover code removed (Jul 2026), and Homebrew
7.0.11 on this Mac **rejects the flag** while still quarantining downloads by
default (verified on our own epher dmg). Our `xattr` advice still works; our
`HOMEBREW_CASK_OPTS` advice is dead on current brew.

## Goal

Decide whether the mac-brew-hint on epher.org still matches (a) what users are
told elsewhere, (b) Apple's current first-launch flow on macOS 15 Sequoia and
later, and (c) Homebrew's documented cask policy — and whether any part of it
is now stale, unique, or worth revisiting. Observations only; nothing on the
site was touched.

## Method (all claims verified)

Everything below was fetched directly on 2026-11-01 unless marked UNVERIFIED:

- **Vendor pages**: gimp.org, krita.org, kdenlive.org, handbrake.fr (docs +
  downloads.php), mac.getutm.app / getutm.app / docs.getutm.app (the full docs
  repo source, 111 files), openemu.org (via its GitHub site repo, release
  notes included), mos.caldis.me, maccy.app, keka.io, rectangleapp.com,
  raycast.com, videolan.org (download-macosx + news.html), iterm2.com, plus
  GitHub READMEs and release pages for OpenEmu, Mos, AltTab, Stats, LocalSend,
  Maccy, Rectangle, OBS, IINA, AltTab [1]–[32].
- **Vendor source repos** where sites are JS-rendered or curl-blocked:
  `arduino/docs-content` (the markdown behind docs.arduino.cc),
  `utmapp/docs.getutm.app`, `ghostty-org/ghostty.org`, `OpenEmu/OpenEmu.github.io`,
  `lwouis/alt-tab-macos` [8][11][24][14][17][18].
- **Homebrew**: docs.brew.sh Cask Cookbook, Acceptable Casks, FAQ, Security and
  Supply-Chain; blog posts 5.0.0/6.0.0/7.0.0; PRs #4656 (2018, quarantine
  default), #20775 (2025, warning strengthened), #20929 (2025, deprecation);
  removal/inheritance commits [33]–[44]. Wayback snapshots of Acceptable-Casks
  (May 2025) for the pre-policy text [45].
- **This Mac (primary evidence)**: Homebrew 7.0.11 — `man brew`, `brew install
  --help`, a live `brew install --cask --no-quarantine` rejection, `brew fetch
  --cask` plus the quarantine xattr on the downloaded artifact (the epher dmg
  itself, from Pete's tap), and `brew info --cask` for all 20 surveyed casks.
- **Apple**: support.apple.com/102445 (fetched in full), the developer
  notarization documentation (JSON API), and an attempt at discussions.apple.com
  (blocked — see Community) [46][47][52].
- **Independent high-trust secondary**: four eclecticlight.co articles by
  Howard Oakley, including his controlled experiments on fresh macOS VMs [48]–[51].
- Known fetch failures, honestly: inkscape.org and iina.io return 403 to curl
  (Inkscape's release page read via a Wayback snapshot instead; IINA's FAQ has
  no Wayback captures at all); ghostty.org/docs and docs.arduino.cc are JS
  shells to curl (both checked via their source repos); discussions.apple.com
  serves a "Security Verification" wall to curl.

## Survey: 20 Homebrew casks and their Gatekeeper guidance

Cask existence verified locally against the current brew API (`brew info --cask`,
2026-11-01). "Notarized?" reflects what the vendor's own fetchable sources
state; where I could not verify it, it is marked UNVERIFIED rather than guessed.
Remedy categories: (i) right-click/Control-click → Open, (ii) System Settings →
Privacy & Security → Open Anyway, (iii) xattr quarantine removal, (iv)
`--no-quarantine`/`HOMEBREW_CASK_OPTS`, (v) App Store instead, (vi) nothing.

| App (cask) | Notarized? | Site guidance | Remedy | Source |
|---|---|---|---|---|
| GIMP (`gimp` 3.2.6) | UNVERIFIED | (vi) — download page checks SHA256 + VirusTotal only; no Gatekeeper text | (vi) | [1] |
| Inkscape (`inkscape` 1.4.4) | UNVERIFIED | (vi) — site 403s curl; 1.4 release page read via Wayback, no signing/Gatekeeper text | (vi) | [2] |
| Krita (`krita` 5.3.4.1) | UNVERIFIED (DMG named `krita-5.3.4.1-signed.dmg`) | (vi) — download page and docs installation page have no Gatekeeper text | (vi) | [3][4] |
| Kdenlive (`kdenlive` 26.08.2) | UNVERIFIED | (vi) — macOS section says only "Requires macOS 13 (Ventura) or newer" | (vi) | [5] |
| HandBrake (`handbrake-app` 1.11.2) | UNVERIFIED | (vi) — docs and downloads.php push checksum verification, no Gatekeeper text | (vi) | [6][7] |
| Arduino IDE (`arduino-ide` 2.3.10) | UNVERIFIED | (vi) — install doc: "To install the Arduino IDE 2 on a macOS computer, simply copy the downloaded file into your application folder."; Help-Center search for gatekeeper/malicious/quarantine returns nothing | (vi) | [8][9] |
| UTM (`utm` 5.0.8) | UNVERIFIED | (vi) — product site and the entire docs source have zero Gatekeeper/quarantine mentions | (vi) | [10][11] |
| IINA (`iina` 1.5.0) | UNVERIFIED | (vi) — iina.io 403s curl and its FAQ has no Wayback captures; GitHub releases page silent | (vi) | [12] |
| OpenEmu (`openemu` 2.4.1) | UNVERIFIED | Mentions Gatekeeper as engineering notes, no user remedy: "Improved UX to mitigate game launch issues related to Gatekeeper. — Now prompts with requirement to move to your Applications folder on app launch." | anti-App-Translocation UX, not a quarantine remedy | [13][14] |
| Mos (`mos` 4.2.1) | UNVERIFIED | (vi) — README/website silent for users; only a contributor note that changes touching "signing, notarization" carry higher risk | (vi) | [15][16] |
| AltTab (`alt-tab` 11.10.1) | **Yes** — repo doc: "AltTab parks its stapled notarization ticket while it runs and puts it back when it quits." | (vi) — README has zero Gatekeeper mentions | (vi) | [17][18] |
| Stats (`stats` 3.0.24) | UNVERIFIED | (vi) — README silent | (vi) | [19] |
| LocalSend (`localsend` 1.18.2) | UNVERIFIED | (v)-shaped — README's macOS install row offers App Store and Homebrew; no Gatekeeper text | (v) by install-table placement | [20] |
| Maccy (`maccy` 2.8.0) | UNVERIFIED | (vi) — README + maccy.app silent (their "malware" text warns about fake Maccy websites) | (vi) | [21][22] |
| Ghostty (`ghostty` 1.4.0) | UNVERIFIED | (vi) — docs site is JS-rendered; the docs source repo and the author's announcement post have zero Gatekeeper/notarization mentions | (vi) | [23][24][25] |
| iTerm2 (`iterm2` 3.7.6) | **Yes** — changelog: "It is also notarized so it will run on Catalina." | (vi) — states the fact, no user guidance | (vi) | [26] |
| Rectangle (`rectangle` 2.0.4) | UNVERIFIED | (vi) — site and README silent | (vi) | [27] |
| Raycast (`raycast` 2.10.1) | UNVERIFIED | (vi) — site silent (JS shell to curl) | (vi) | [28] |
| VLC (`vlc` 3.0.24) | UNVERIFIED | (vi) — mac download page and news archive have no Gatekeeper/notarization text | (vi) | [29][30] |
| OBS (`obs` 33.0.1) | UNVERIFIED | (vi) — releases pages (HTML, 100 releases checked via API page) contain no notarization text | (vi) | [31] |

(Surrounding checks: Keka — also silent [32]. Across all 20 casks, zero
instances of remedies (i)–(iv) in vendor guidance.)

## Homebrew's documented position: from "at your own risk" to a Gatekeeper requirement

**The old world.** Quarantine of cask downloads became the default when brew
absorbed Cask: PR #4656 "Enable quarantining of Homebrew-Cask's downloads" was
merged 2018-08-31 (commit `f7cafb50ad`) [41]. Until late 2025, Homebrew's
Acceptable Casks page was explicitly hands-off: "Homebrew Cask has long decided
it will not be an active gatekeeper (macOS already has one) and users are
expected to know about the software they are installing" and "If an app that
bundles malware was not signed with an Apple Developer ID and you purposefully
disabled or bypassed Gatekeeper, no action will be taken on our part. When you
disable security features, you do so at your own risk." Unsigned apps were only
rejected at audit if they "fail with GateKeeper enabled on Homebrew supported
macOS versions and platforms (e.g. unsigned apps will not launch on Apple
Silicon Macs)" — all quoted from the Wayback capture of 2025-05-18 [45].

**The reversal — Homebrew 5.0.0, 12 Nov 2025** [37]:

> "Casks without codesigning are deprecated. We will disable all
> Homebrew/homebrew-cask casks that fail Gatekeeper checks in September 2026."
>
> "--no-quarantine and --quarantine flags have been deprecated as Homebrew does
> not wish to easily provide circumvention to macOS security features."

The lead-up is visible in the repo: PR #20775 (2025-09-29) strengthened the
warning shown to `--no-quarantine` users [42]; PR #20929 "Prepare for
deprecation of `--no-quarantine`." merged 2025-10-23 [40]. **Homebrew 6.0.0**
(11 Jun 2026) confirmed enforcement: "Casks that fail macOS Gatekeeper checks,
deprecated in 5.0.0, remain on track to be disabled in September 2026" [38];
whether the disable then actually ran is UNVERIFIED (the 7.0.0 post, 13 Sep
2026, mentions no Gatekeeper items [39]).

**Current policy** (all fetched 2026-11-01): the Cask Cookbook states
"Official macOS casks must also meet the Gatekeeper requirement" [33];
Acceptable Casks requires that "apps, installers and other executable artefacts
that Gatekeeper can assess must pass Homebrew's Gatekeeper checks and must not
require System Integrity Protection or Gatekeeper to be disabled or bypassed"
[34]; the FAQ now answers "Why was a cask disabled or removed after a macOS
security check?" with "Official macOS casks must pass Homebrew's Gatekeeper
checks without requiring users to bypass them. A failure is not a malware
verdict, but the cask may be deprecated, disabled and eventually removed until
upstream provides artefacts that pass" [35]; and the Security and Supply-Chain
page explains the asymmetry — casks are "prebuilt applications and installers
supplied directly by the upstream developer" whose sha256 "does not prove who
produced those bytes" [36].

**Verified on this Mac (Homebrew 7.0.11, 2026-11-01)**:

- `brew install --cask --no-quarantine <name>` errors out with a Usage message —
  the flag is rejected outright.
- `man brew` contains zero occurrences of quarantine, Gatekeeper or
  notarization; `HOMEBREW_CASK_OPTS` is still documented but only for `--*dir`
  options, `--language`, `--require-sha` and `--no-binaries`.
- Quarantine is still applied by default: the epher dmg in
  `~/Library/Caches/Homebrew/downloads/` (fetched by brew from Pete's tap)
  carries `com.apple.quarantine` [53][54].
- Code-level: the leftover `--no-quarantine` code was removed 2026-07-30
  (`ba25213c81`) [44]; and a 2026-07-11 change ("Preserve cask quarantine
  approval") makes brew inherit a user's Gatekeeper approval across cask
  upgrades "only when the new app satisfies the old app's designated
  requirement" [43] — relevant to epher, whose ad-hoc-signed app has no stable
  designated requirement to satisfy (see Implications).

One scope note: our cask lives in Pete's third-party tap (`upyesp/tap`) [54],
not official `homebrew/cask`, so the official-cask disable policy does not
directly apply to it — but the flag removal and quarantine defaults apply to
every brew user regardless of tap.

## Apple's current guidance, and the Sequoia change

**Apple Support HT102445, "Safely open apps on your Mac"** (fetched in full
2026-11-01) documents exactly one override flow [46]:

> "If you want to open an app that hasn't been notarized or is from an
> unidentified developer … After you've tried to open the app, follow these
> steps: Open System Settings. Click Privacy & Security, scroll down, and click
> the Open Anyway button to confirm your intent to open or install the app. The
> warning prompt reappears and, if you're absolutely sure that you want to open
> the app anyway, you can click Open. The app is now saved as an exception to
> your security settings, and you can open it in the future by double-clicking
> it, just as you can any authorized app."

Two more load-bearing sentences from the same page: "Check the App Store for an
updated version or search for an alternative app" is Apple's primary advice for
an app whose developer "can't be verified"; and the page notes macOS Catalina
and later "requires software to be notarized" by default.

**On the Sequoia change (the critical check):** the phrase Control-click → Open
appears nowhere on the current page (zero occurrences in the fetched HTML);
Apple's only documented bypass is Open Anyway. That matches Howard Oakley's
controlled reporting from the Sequoia betas: in "Gatekeeper and notarization in
Sequoia" (2024-08-10) he describes the old flow — "the user has to select the
Open command from the Finder's contextual menu" — and the new one: "Gatekeeper
will then refuse to run for the first time those apps that aren't notarized, if
they're in quarantine. The only way that you'll then be able to launch an
unnotarized app is by adding it to a list of exemptions using Privacy &
Security settings. That's performed on an individual basis, per app" [48]. In
"Living with(out) notarization" (2024-10-01) he walks the new Open Anyway flow
including the repeated warning and admin authentication, and shows why it
exists: "The old Finder bypass for Gatekeeper was simple: use the contextual
menu to Open the app, and there's a single dialog to get through. That's so
simple that it has become widely used by malware" — illustrated with Atomic
Stealer's own right-click-Open instructions [49]. I found no Apple statement
explicitly announcing removal of the contextual-menu bypass; per Apple's page
and Oakley's testing it is absent from Sequoia's documented flow, and I mark
whether it still silently works on some builds as UNVERIFIED. Oakley's
comment-thread testing adds a 15.1-era nuance directly relevant to epher:
ad-hoc-signed apps still behave as before (Open Anyway works); it is a damaged
or missing signature that breaks an app outright, so our ad-hoc signing is not
itself the blocker — the missing notarization is [49].

**Where the defaults stand in 2026** — from Oakley's later experiments on fresh
macOS VMs: an unquarantined app "launched normally, without any warning dialog"
even when not notarized ("If an app doesn't have a quarantine extended
attribute … the app is allowed to run without any additional action" [48][51]);
in Tahoe 26.6.1 a quarantined ad-hoc-signed app runs after the "Anywhere"
setting and a confirmation dialog [50][51]; and genuinely unsigned or
broken-signed quarantined apps are invited to the Trash [51]. Apple's
developer-side expectation is unchanged: "Give users even more confidence in
your macOS software by submitting it to Apple for notarization" [47].

## Forum/community evidence (community/anecdotal, not Apple policy)

Apple's own discussion forums are closed to curl (the search URL returns a
"Security Verification" page), so no thread quotes could be verified — marked
UNVERIFIED [52]. The nearest verifiable substitute is the practitioner
discussion under Oakley's articles [49]: developers report the pre-Sequoia
burden ("Before Sequoia this wasn't too much of an issue, sure you had to tell
people to right click open, but Sequoia's new warnings are much more likely to
scare off potential users, and the only fix is to fork over $99 every year");
Oakley counters that notarization took him "all of 5 minutes"; one commenter
documents the open-source app OpenSCAD having just gone through signing and
notarization "not without consternation"; and Oakley's own advice to users is
against stripping quarantine ("You shouldn't remove the quarantine xattr
anyway: if that code hasn't been run on that Mac before it does need to be
checked"). Treat the tone of that thread — hobbyist vendors squeezed, users
confused by the dialog's wording — as the community consensus, not Apple policy.

## What unsigned vendors typically say

Almost nothing. Of 20 popular casks, 18 publish no Gatekeeper guidance at all
(remedy (vi)), one steers to the App Store by install-table placement, and one
mentions Gatekeeper only as an engineering note about App Translocation [1]–[32].
Nobody recommends right-click → Open any more — the remedy most vendors used to
give pre-Sequoia now exists mainly in malware instructions [49]. The two apps
I could verify as notarized (iTerm2, AltTab) mention notarization as a release
fact or an internal stapling detail, never as user guidance [18][26]. The
practical consequence for any unsigned vendor: whatever first-run advice
exists, it comes from you, not from Apple's dialog (which offers only Done /
Move to Trash and steers to the App Store) [46] and not from Homebrew, which
now treats a bypass as something it "does not wish to easily provide" [37].

## Implications for epher.org (observations, not instructions)

Our current hint (verbatim, `site/i18n/en.js`) [55]:

> "epher isn't notarized with Apple, so the first launch may show an "Apple
> could not verify" warning. Choose Done, then run xattr -rd
> com.apple.quarantine /Applications/epher.app once to clear it. The warning
> returns after brew upgrade; exporting HOMEBREW_CASK_OPTS="--no-quarantine" in
> your shell profile stops it for good (for Homebrew apps)."

1. **Our advice is now unique in the ecosystem.** No surveyed vendor publishes
   xattr or `--no-quarantine` guidance (0 of 20); we are the only voice telling
   brew users what to do. That makes accuracy entirely our problem — and it
   makes the next point material.
2. **The "durable fix" no longer works on current brew.** `--no-quarantine` was
   deprecated in Homebrew 5.0.0 (Nov 2025) [37], its leftover code removed in
   Jul 2026 [44], and brew 7.0.11 on this Mac rejects it at the CLI; the man
   page no longer lists it among supported `HOMEBREW_CASK_OPTS` [53]. Users who
   set the env var per our note will get no quarantine relief (CLI path
   verified rejected; env-var path not end-to-end tested without installing —
   UNVERIFIED but almost certainly inert). The sentence should eventually be
   removed or rewritten; leaving it teaches a habit Homebrew has deliberately
   closed off.
3. **The xattr advice still works — and remains the only automatable path.**
   Oakley's experiments confirm unquarantined apps run without prompts even
   unnotarized [48][51], and brew still quarantines every download by default
   (verified on our own dmg) [53]. But note the rhythm: quarantine returns on
   every `brew upgrade` (fresh download each time), so "run once" is really
   "run after each upgrade" — which our hint already says. Homebrew's
   approval-inheritance change of 2026-07-11 only survives upgrades "when the
   new app satisfies the old app's designated requirement" [43]; an ad-hoc
   signature has no stable designated requirement, so (analysis, not
   test-verified) Open Anyway-style approvals won't persist across our
   upgrades either.
4. **Apple's blessed alternative is absent from our note.** System Settings →
   Privacy & Security → "Open Anyway" is the only remedy Apple documents [46],
   it requires no shell command, and the exception persists for future
   double-click launches. Vendors don't mention it either — but our hint's
   Done-then-xattr flow could sit alongside an Open Anyway mention (after the
   first failed launch, which is what primes the button) without contradicting
   anything.
5. **We never gave the stale remedy.** Right-click → Open — the guidance
   everyone used to publish, and which post-Sequoia is both wrong and, per
   Oakley, the malware playbook [49] — never appears in our copy. Our note
   matched the dialog reality from the start: on Sequoia+ the alert only offers
   Done / Move to Trash, exactly as our "Choose Done" line assumes.
6. **The dialog actively talks users out of us.** Apple's page tells people
   facing an unverifiable developer to "Check the App Store for an updated
   version or search for an alternative app" [46]. Our pre-emptive hint is the
   only counterweight users get; dropping it (e.g. if the brew hint ever feels
   too long) would leave every cask user alone with a dialog whose only in-dialog
   option besides Done is Move to Trash.
7. **Homebrew's policy direction is a signal worth tracking.** Official casks
   that fail Gatekeeper checks face deprecation → disable [34][35][38]. Our
   cask sits in a third-party tap, so enforcement doesn't reach it — but the
   ecosystem expectation (notarized artifacts, no-bypass defaults, quarantines
   you can no longer opt out of) is now uniform across Apple and Homebrew. The
   shelved notarization work remains the only path to a genuinely quiet
   first launch; until then, xattr-after-upgrade is the working automation and
   Open Anyway is the working GUI path.

## Sources

All fetched 2026-11-01 unless noted. UNVERIFIED items are flagged inline above.

1. GIMP downloads (GIMP for macOS section): https://www.gimp.org/downloads/
2. Inkscape 1.4 release page (site 403s curl; read via
   https://web.archive.org/web/2026id_/https://inkscape.org/release/inkscape-1.4/):
   https://inkscape.org/release/inkscape-1.4/
3. Krita download (macOS installer `krita-5.3.4.1-signed.dmg`): https://krita.org/en/download/
4. Krita manual, Installation: https://docs.krita.org/en/user_manual/getting_started/installation.html
5. Kdenlive download (macOS section): https://kdenlive.org/download/
6. HandBrake docs, Downloading and installing: https://handbrake.fr/docs/en/latest/get-handbrake/download-and-install.html
7. HandBrake downloads.php: https://handbrake.fr/downloads.php
8. Arduino IDE install tutorial, markdown source (docs.arduino.cc is a JS shell
   to curl): https://github.com/arduino/docs-content/blob/main/content/software/ide-v2/tutorials/getting-started/01.ide-v2-downloading-and-installing/ide-v2-downloading-and-installing.md
9. Arduino Help Center article 360019833020 "Download and install Arduino IDE"
   (fetched via https://support.arduino.cc/api/v2/help_center/en-us/articles/360019833020.json;
   gatekeeper/malicious/quarantine searches return nothing):
   https://support.arduino.cc/hc/en-us/articles/360019833020
10. UTM product site: https://mac.getutm.app/ and https://getutm.app/
11. UTM docs source repo (111 files, zero Gatekeeper/quarantine mentions):
    https://github.com/utmapp/docs.getutm.app
12. IINA: iina.io/faq/ 403s curl with no Wayback captures
    (https://web.archive.org/web/2026*/iina.io/faq/ → none); releases page:
    https://github.com/iina/iina/releases
13. OpenEmu README: https://github.com/OpenEmu/OpenEmu/blob/master/README.md
14. OpenEmu release notes 2.3.1/2.4 (source repo OpenEmu/OpenEmu.github.io,
    rnotes/2.3.1.html, rnotes/2.4.html): https://github.com/OpenEmu/OpenEmu.github.io
15. Mos README (en + zh): https://github.com/Caldis/Mos/blob/master/README.md and README.enUS.md
16. Mos website: https://mos.caldis.me/
17. AltTab README (zero Gatekeeper mentions): https://github.com/lwouis/alt-tab-macos/blob/master/README.md
18. AltTab `src/util/StapledTicketSpecs.md` (stapled notarization ticket):
    https://github.com/lwouis/alt-tab-macos/blob/master/src/util/StapledTicketSpecs.md
19. Stats README: https://github.com/exelban/stats/blob/master/README.md
20. LocalSend README (macOS install row): https://github.com/localsend/localsend/blob/main/README.md
21. Maccy README: https://github.com/p0deje/Maccy/blob/master/README.md
22. Maccy site: https://maccy.app/
23. Ghostty docs (JS shell to curl): https://ghostty.org/docs/faq
24. Ghostty docs/site source repo (zero Gatekeeper mentions): https://github.com/ghostty-org/ghostty.org
25. Ghostty announcement (zero Gatekeeper/notarization mentions): https://mitchellh.com/announcing-ghostty
26. iTerm2 downloads/changelog pages ("It is also notarized so it will run on
    Catalina"): https://iterm2.com/downloads.html
27. Rectangle: https://rectangleapp.com/ and https://github.com/rxhanson/Rectangle/blob/master/README.md
28. Raycast: https://www.raycast.com/
29. VLC macOS download page: https://www.videolan.org/vlc/download-macosx.html
30. VLC news archive: https://www.videolan.org/news.html
31. OBS releases (HTML + API pages, no notarization text): https://github.com/obsproject/obs-studio/releases
32. Keka: https://www.keka.io/en/
33. Homebrew Cask Cookbook ("must also meet the Gatekeeper requirement"): https://docs.brew.sh/Cask-Cookbook
34. Homebrew Acceptable Casks, "Platform compatibility and macOS security
    protections": https://docs.brew.sh/Acceptable-Casks
35. Homebrew FAQ, "Why was a cask disabled or removed after a macOS security
    check?": https://docs.brew.sh/FAQ
36. Homebrew Security and Supply-Chain, "Casks have a different trust model":
    https://docs.brew.sh/Homebrew-Security-and-Supply-Chain
37. Homebrew 5.0.0 announcement (12 Nov 2025; codesigning deprecation,
    quarantine-flag deprecation): https://brew.sh/2025/11/12/homebrew-5.0.0/
38. Homebrew 6.0.0 announcement (11 Jun 2026; failing casks disabled Sep 2026):
    https://brew.sh/2026/06/11/homebrew-6.0.0/
39. Homebrew 7.0.0 announcement (13 Sep 2026; no Gatekeeper items): https://brew.sh/2026/09/13/homebrew-7.0.0/
40. Homebrew/brew PR #20929 "Prepare for deprecation of `--no-quarantine`."
    (merged 2025-10-23): https://github.com/Homebrew/brew/pull/20929
41. Homebrew/brew PR #4656 "Enable quarantining of Homebrew-Cask's downloads"
    (merged 2018-08-31; commit f7cafb50ad): https://github.com/Homebrew/brew/pull/4656
42. Homebrew/brew PR #20775 "strengthen `--no-quarantine` message"
    (2025-09-29): https://github.com/Homebrew/brew/pull/20775
43. Commit 0a137ee80b "Preserve cask quarantine approval" (2026-07-11):
    https://github.com/Homebrew/brew/commit/0a137ee80b
44. Commit ba25213c81 "Remove leftover code for `--no-quarantine`" (2026-07-30):
    https://github.com/Homebrew/brew/commit/ba25213c81
45. Acceptable Casks, pre-policy text (Wayback 2025-05-18 snapshot; "not an
    active gatekeeper … at your own risk"): https://web.archive.org/web/20250518040441/https://docs.brew.sh/Acceptable-Casks
46. Apple Support, "Safely open apps on your Mac" (HT102445): https://support.apple.com/en-us/102445
47. Apple Developer, "Notarizing macOS software before distribution":
    https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
48. Howard Oakley, "Gatekeeper and notarization in Sequoia" (2024-08-10):
    https://eclecticlight.co/2024/08/10/gatekeeper-and-notarization-in-sequoia/
49. Howard Oakley, "Living with(out) notarization" (2024-10-01), incl. Open
    Anyway flow, Atomic Stealer's right-click-Open instructions and the
    comment-thread testing of 15.1: https://eclecticlight.co/2024/10/01/living-without-notarization/
50. Howard Oakley, "Can you disable Gatekeeper and XProtect?" (2026-08-06):
    https://eclecticlight.co/2026/08/06/can-you-disable-gatekeeper-and-xprotect/
51. Howard Oakley, "How can you run code that hasn't been notarised?"
    (2026-08-11; fresh-VM experiments): https://eclecticlight.co/2026/08/11/how-can-you-run-code-that-hasnt-been-notarised/
52. Apple Discussion forums search (curl hits "Security Verification";
    UNVERIFIED): https://discussions.apple.com/search?q=%22apple%20could%20not%20verify%22
53. Local primary evidence, this Mac, 2026-11-01: Homebrew 7.0.11 (`brew
    --version`), `man brew` (no quarantine/Gatekeeper text; HOMEBREW_CASK_OPTS
    without --no-quarantine), `brew install --cask --no-quarantine` rejection,
    and `com.apple.quarantine` present on the brew-downloaded epher dmg in
    ~/Library/Caches/Homebrew/downloads/.
54. epher cask (third-party tap upyesp/tap; local `brew info --cask epher`):
    https://github.com/upyesp/homebrew-tap/blob/HEAD/Casks/epher.rb
55. Our current site copy, repo file site/i18n/en.js, key `mac-brew-hint`
    (read for comparison only; not modified):
    https://github.com/upyesp/epher/blob/staging/site/i18n/en.js
