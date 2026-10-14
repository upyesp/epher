# Flathub's Generative AI policy, the accepted AI-assisted apps, and what our rejected submission actually tripped on

Research of 2026-10-14, in preparation for a possible re-submission of the
Flathub listing (PR flathub/flathub#10096, closed 2026-09-06 with the
"AI Slop" label). Three questions: what exactly happened to our submission,
what Flathub's policy on AI-developed apps says today, and which AI-built
apps have been accepted — with a plan for how to proceed if the listing
comes back on the roadmap.

## Goal

Decide whether and how a second Flathub submission could pass: the
mechanical rules our first PR broke, the disclosure obligations the
current policy carries, the reviewer behaviour the accepted and rejected
submissions reveal, and the honest budget a re-submission asks for
(ADR-0061's amendment dropped Flathub for exactly that budget).

## Method (all claims verified)

Everything below was fetched directly on 2026-10-14, no blog posts:

- **The rejection itself**: the GitHub API records of PR #10096 (body,
  timeline events, bot comment) and its diff [1][2][3].
- **The policies**: docs.flathub.org (submission guide, requirements
  incl. the Generative AI policy section) [4][5].
- **The policy history**: the flathub-infra/documentation git log and the
  diffs of the relevant commits (2025-06-19 "Add a policy to stop LLM
  slops", 2026-05-28 blanket-ban rewording, 2026-09-04 disclosure-based
  replacement), plus the discussion on PR #641 and issue #620 [6][7][8][9].
- **The label**: flathub/flathub's label list ("AI Slop: Mark obviously AI
  generated PR or app.") and a label search (1,226 items) [10][11].
- **Accepted and rejected AI-assisted submissions**: PRs #8030 (Speed of
  Sound), #10333 (Alexandria), #10270 (Lightning), the Fog Panther
  Discourse thread #11888 and its live repo/listing, the PyTrain
  Discourse thread #12927, the Dealer's Choice thread #12357, and
  existence checks of the flathub/<app-id> repositories [12]–[22].

## What happened to PR #10096, minute by minute

The PR ("epher: add com.epher.Desktop", 4 files, 110 added lines) went up
at 11:08:58 UTC on 2026-09-06. The submission-checker bot closed it 27
seconds later: "does not follow the submission guidelines … Diagnostics:
**Files not in toplevel**" — the manifest, desktop file, metainfo, and
icon were nested in a `com.epher.Desktop/` directory, while Flathub wants
them at the repository toplevel of the submission branch [1][2]. The
title also missed the required `Add $FLATPAK_ID` shape, the template
checklist was absent (replaced by a free-form summary), and there was no
showcase video [2][4].

Seventy-five minutes after the close, moderator CodedOre applied the
"AI Slop" label ("Mark obviously AI generated PR or app.") [1]. The
closure was mechanical; the label was the human judgment, and the diff
explains it: every file carried the project's narrator comments citing
ADR numbers and release-workflow internals (even the SVG carries a
variant-history essay), the metainfo's only release entry was the
placeholder `version="0.0.0"`, and the GNOME runtime was pinned at 46 —
two generations behind. That is the same texture barthalion later called,
in another review, "slop vomit, full of irrelevant comments no human
being would write themselves" [15]. And the PR carried no AI disclosure
under a disclosure policy that had taken effect two days earlier [8].

Anyone clicking through to the upstream repo found the other prime: a
commit wall in the agent register ("the train versions move to 0.5.86").
The label sorts the current wave — 1,226 PRs and issues carry it [11].

## The policy, and how it got there

Timeline from the documentation repo's history [6][7][8]:

- **2025-06-19** — "Add a policy to stop LLM slops": scrutiny of structure
  and organization as an inclusion criterion.
- **2026-01-24** — a more detailed gen-AI policy.
- **2026-05-28** — the blanket ban, verbatim: "Applications containing
  AI-generated or AI-assisted code, documentation, or other content are
  not allowed … Exceptions may be granted for mature, well-maintained
  projects."
- **2026-09-04** — PR #641 replaces the ban with the disclosure-based
  policy now in force. The stated motivation (barthalion): "keep the door
  open for LLM-assisted apps while giving reviewers a simple way of
  rejecting obvious slop"; the ban had caused "downstream waves" (GNOME
  Circles, Universal Blue) and, worse, kept legitimate authors from
  submitting at all [9].

The current Generative AI policy (requirements → Generative AI policy) [5]:

1. **Disclosure is mandatory** for any AI-generated code, documentation,
   packaging, or other material the submitter knows or reasonably
   believes is in the application or its Flathub packaging, identifying
   the affected parts and approximate extent.
2. **Flathub manifests must not contain AI-generated or AI-assisted
   content.** Disclosure does not exempt manifests.
3. AI used only for research, discussion, or debugging — with no
   generated material shipped — needs no disclosure. Disclosed material
   is evaluated at reviewer discretion; reviewers may reject, including
   without further review, based on the extent or role of generated
   material or concerns about its review, quality, or maintainability.
   Disclosure creates no presumption of acceptance.
4. **AI tools or agents must not open or automate the submission PR, or
   generate its commit messages, descriptions, review comments, or
   replies. Submitters must not request AI-agent reviews.**
5. Undisclosed or materially misrepresented material, or prohibited
   interactions, mean rejection; repeated violations mean a permanent
   ban.

The surrounding rules that bite: PRs that drop the template, break the
guidelines, or "contain excessive AI-generated content may be closed
without a review" [4]; inclusion criteria demand meaningful development
history, real-world use, and ongoing maintenance, and stable (not
pre-release) versions [5]; submissions may be rejected at any stage or
recalled post-merge, and prior conduct counts ("Trust and history") [4][5].

Reviewer mood, in their own words: barthalion — "no slop, legit projects
with some LLM usage probably fine" [9]; travier — submission PRs and all
their interactions "should NOT be AI generated at all" [9]; teamcons
(Discourse, 2026-10-11) — "How much AI we can tolerate varies slightly
between reviewers, but overall we see it as a liability … We want to have
your app, with your code, in, not the silicon valleys. Inclusion can be
considered if there is honest work to limit it in the workflow to the
absolute minimum and rework the tainted parts" [19].

## The accepted AI-assisted apps

- **Speed of Sound** (io.speedofsound.SpeedOfSound, PR #8030, accepted
  2026-03-25). A voice-typing app developed with a coding agent (a
  CLAUDE.md rides in its commits). A reviewer flagged "It seems
  vibecoded"; the author admitted the agent use, pointed at a candid
  public FAQ on exactly that question, then spent two and a half weeks
  personally answering every review point with real fixes. Accepted and
  verified. The model case: human at the interface, honest about the
  machinery [12][13].
- **Alexandria** (io.github.pemsley.Alexandria, PR #10333, accepted
  2026-09-28). A PDF/reference manager by Coot's author, submitted under
  the current policy with the disclosure "a substantial part of the work
  is being done with help from coding agents like ClaudeCode for
  efficiency". Hand-written manifest and PR, real upstream history —
  merged by barthalion. Proof the disclosure path works [14].
- **Lightning** (org.lightning_matrix.Lightning, PR #10270, accepted
  2026-09-23). A Matrix client disclosing AI in tests, docs, parts of
  C++/QML/Rust, and CI. It nearly died: the manifest was full of
  AI-narrator comments ("slop vomit") and its PR replies looked
  agent-written (a stray "/review" gave it away). The author hand-cleaned
  the manifest and it merged. Shows both the red line and that it is
  recoverable [15].
- **Fog Panther** (com.fogpanther.FogPanther, accepted 2026-03-18). A
  GIMP-like editor widely suspected to be substantially AI-built;
  verified and still listed despite community reports. A pre-ban-era
  admission that would not clear today's bar — a cautionary inclusion,
  not a model [16][17].

The names kept out are as informative: lemonade-sdk (Claude as its top
committer, ~130 commits, ~90k lines) and several heavily generated
flatpark.org apps were cited in #641 as the kind that would not pass [9];
a well-regarded PyTrain developer was told in October that a codebase
reliant on AI is "not super bueno", and took the hint [19]. Dealer's
Choice, a mostly vibe-coded poker game, was advised that a hand-made
manifest and hand-made PR "might be able to argue it's well maintained"
— and has not surfaced on Flathub since [20].

The pattern across acceptances: disclosure plus hand-written packaging
plus genuinely human review interactions plus sustained project history
plus responsive iteration. The rejections: missing disclosure, slop
artifacts, agent-touched PR interactions.

## Plan for a re-submission

**Step 0 — decide (ADR-level).** ADR-0061's amendment (2026-09-18) dropped
Flathub for budget reasons — review latency plus a per-release bump PR in
a Flathub-owned repo — and says revisiting is a new decision. It also
records that "the Flathub submission never went out", which is wrong:
PR #10096 exists and was closed; the ADR deserves a correction when this
reopens. Flathub is not a distribution necessity (deb, rpm, AppImage, the
apt/dnf repo, Snap, and the release-attached `.flatpak` bundles all live);
only proceed if the listing earns its budget back.

**Step 1 — de-slop the artifacts, by human hands.** Rewrite the submission
manifest from scratch by hand: no narrator comments, no ADR
cross-references in Flathub files; current runtime (51); a real version
tag; files at the toplevel; title `Add org.epher.Desktop`; the full
template. Pick the app ID deliberately and permanently (the PR used
`com.epher.Desktop`, the in-repo manifest now says `org.epher.Desktop`;
`org.epher` matches epher.org and enables verification). Metainfo with
real releases and guideline screenshots; `flatpak-builder-lint` and a
local `org.flatpak.Builder` build green before the PR — Flathub's CI is
not a dev loop (Lightning's author got told so in one line). Record the
required showcase video of the app running as a Flatpak.

**Step 2 — write the disclosure honestly.** The template's AI checkbox is
where this is won or lost: an accurate inventory with extents, e.g. the
application code developed with heavy AI-agent use under human direction,
packaging and PR written by hand. Reviewers demonstrably accept disclosed
agent-assisted code (Alexandria, Lightning); undisclosed material risks
rejection and a ban. Every PR comment, reply, and commit message is typed
by the human. No agent opens, builds on, or comments in that thread.

**Step 3 — preempt the click-through.** Reviewers will land on the upstream
repo and meet the agent-styled commit history. History stays (rewriting it
would be dishonest and worse); the commit register goes human from here
on, and the PR description carries two or three plain sentences in the
maintainer's own voice about the project's history and what the first
submission got wrong. Acknowledge the prior label; do not argue with it —
"Trust and history" cuts both ways.

**Step 4 — submit and staff it.** A fresh PR against `new-pr` (never a
reopen of #10096), one named human owning it end-to-end until merge,
expecting volunteer-reviewer latency and iterative asks (Lightning took
six days, Alexandria one week, Speed of Sound two and a half). After the
merge, the maintenance cost the amendment rejected resumes: bump PRs, the
External Data Checker, protected master.

Honest odds: plausible, not assured. The mechanical failure is trivial to
fix; the perception is fixable only with genuinely human packaging,
candor in the disclosure, and human engagement in the thread. The
disclosure-based window is real and recent — Alexandria and Lightning got
through it in the same month this was written — but reviewer discretion
is wide and mood varies by reviewer.

## Sources

All fetched 2026-10-14 unless marked as repo files.

1. PR #10096 timeline (created 11:08:58Z, bot comment 11:09:25Z, label by
   CodedOre 12:24:49Z): https://github.com/flathub/flathub/pull/10096 and
   https://api.github.com/repos/flathub/flathub/issues/10096/events
2. PR #10096 diff (four files under `com.epher.Desktop/`, narrator
   comments, `version="0.0.0"`, runtime 46):
   https://github.com/flathub/flathub/pull/10096.diff
3. Bot closure comment ("Files not in toplevel"):
   https://api.github.com/repos/flathub/flathub/issues/10096/comments
4. Submission guide (toplevel flow, template warning, "excessive
   AI-generated content may be closed without a review"):
   https://docs.flathub.org/docs/for-app-authors/submission
5. Requirements (Generative AI policy; inclusion and development-history
   criteria; stable releases; trust and history):
   https://docs.flathub.org/docs/for-app-authors/requirements#generative-ai-policy
6. Documentation repo history (2025-06-19 "Add a policy to stop LLM
   slops" 8b1e6588; 2026-01-24 "more detailed gen-AI policy" 2f1a16e9;
   2026-05-28 "Reword LLM policy to make it clear it's not allowed"
   992f57b3; 2026-09-04 "Replace blanket AI ban with disclosure-based
   policy" 43e6e157): https://github.com/flathub-infra/documentation/commits/main
7. Blanket-ban wording (diff of 992f57b3: "Applications containing
   AI-generated or AI-assisted code … are not allowed"):
   https://github.com/flathub-infra/documentation/commit/992f57b3
8. Policy change PR #641 (created 2026-07-07, merged 2026-09-04; new
   disclosure checkbox; barthalion's "keep the door open … rejecting
   obvious slop"; bbhtt's lemonade/flatpark examples; travier's
   human-only interactions): https://github.com/flathub-infra/documentation/pull/641
9. Earlier re-review discussion, issue #620 ("Re-review AI policy to what
   can reasonably be enforced once slop fest is over"):
   https://github.com/flathub-infra/documentation/issues/620
10. Label list ("AI Slop — Mark obviously AI generated PR or app."):
    https://api.github.com/repos/flathub/flathub/labels
11. Label search (total 1,226): https://api.github.com/search/issues?q=repo:flathub/flathub+label:%22AI+Slop%22
12. Speed of Sound PR #8030 (vibecoded remark; CLAUDE.md; FAQ; acceptance
    2026-03-25): https://github.com/flathub/flathub/pull/8030
13. Speed of Sound repo (accepted, live): https://github.com/flathub/io.speedofsound.SpeedOfSound
14. Alexandria PR #10333 (ClaudeCode disclosure; merged 2026-09-28 via
    barthalion /merge; repo created): https://github.com/flathub/flathub/pull/10333
15. Lightning PR #10270 ("slop vomit" manifest; "/review" slip; cleaned
    and accepted 2026-09-23): https://github.com/flathub/flathub/pull/10270
16. Fog Panther repo (created 2026-03-18, still live) and listing:
    https://github.com/flathub/com.fogpanther.FogPanther ,
    https://flathub.org/en/apps/com.fogpanther.FogPanther
17. Fog Panther report thread ("no clear rule blocking AI-written apps"
    at the time; verified status): https://discourse.flathub.org/t/how-do-i-report-an-application-for-being-ai/11888
18. Lightning repo: https://github.com/flathub/org.lightning_matrix.Lightning
19. PyTrain thread (teamcons' tolerance answer; OP concedes):
    https://discourse.flathub.org/t/can-i-add-a-python-app-ive-developed-with-ai-assistance-to-flathub/12927
20. Dealer's Choice thread (razzeee's hand-made-manifest path):
    https://discourse.flathub.org/t/dealers-choice/12357
21. Repo files: `packaging/flatpak/org.epher.Desktop.yml`,
    `packaging/flatpak/org.epher.Desktop.metainfo.xml`,
    `docs/adr/0061-linux-installs-come-from-apt-dnf-flathub-snap-and-aur.md`
    (2026-09-18 amendment: "The Flathub submission never went out" —
    contradicted by [1]).
