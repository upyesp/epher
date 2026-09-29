# Publishing epher.el: MELPA, the de-facto official Emacs channel, and what our single file needs

Research of 2026-09-29, in preparation for distributing `clients/emacs/epher.el`
through the channels Emacs users actually install from. Four questions: what
counts as the "official" channel when Emacs has no first-party store, what a
listing there looks like and renders, what first publication and every later
release train require of a human and of CI, and what the top packages do that
we do not. Like the other publishing research, this document prices the
channel; ADR-0066 still ships Emacs as a ready-made config, not a package.el
plugin.

## Goal

Decide what it takes to get `epher` into MELPA — the recipe, the review, the
exact requirements on `epher.el`'s headers — and whether NonGNU ELPA (the
one GNU-run archive that does not demand copyright assignment) is worth the
second queue.

## Method (all claims verified)

Everything below was fetched directly with curl, no blog posts:

- **MELPA**: the README and CONTRIBUTING.org of `melpa/melpa` at `master`,
  the pull-request template, the CI workflow, and the site's JavaScript
  (`html/js/melpa.js`) — which shows exactly what the package page fetches
  and renders [1][2][3][4][5].
- **package-build** (the builder MELPA runs): `package-build.el` at master,
  read at the function that produces the listing text [6].
- **Live MELPA data**: `https://melpa.org/download_counts.json` (7,259
  package names with download counts) and `https://melpa.org/archive.json`
  (6,325 built packages with versions, descriptions, keywords, authors),
  plus the rendered listing text of the top five packages
  (`https://melpa.org/packages/<name>-readme.txt`) [7][8][9].
- **NonGNU ELPA**: the package index and about page at elpa.nongnu.org, and
  the ELPA "Contributing" page the Contribute! link redirects to (on
  elpa.gnu.org, which governs both archives) [10][11][12].
- **Our file**: `clients/emacs/epher.el` and `clients/emacs/README.md` [13].

## The reality check: two channels, one that matters

Emacs has no first-party store either, but the de-facto official channel is
unambiguous: **MELPA**. It is the largest archive by far (6,325 packages
built right now [8]), it builds packages automatically from upstream repos
via recipes ("a server-side version of el-get, or even Homebrew"), and it is
where the ecosystem's own tooling points by default [1].

The GNU-adjacent official channel is **NonGNU ELPA**: "one of the two default
package archives of GNU Emacs" (with GNU ELPA), default-enabled since Emacs
28.1 [10][11]. Its rules are set on the shared ELPA contributing page: GNU
ELPA requires every contributor to sign the FSF copyright assignment, NonGNU
ELPA does not — "for packages on NonGNU ELPA this is more difficult" to
absorb into Emacs, but "for end-users, this difference is negligible" [12].
It is curated by volunteers on a mailing list, not by PR. Both archives
require a GPL-compatible license (MIT qualifies) [2][12].

Legacy channels (the old Marmalade, ELPA-with-one-package days) are gone;
there is no third queue worth pricing.

## What a listing is on MELPA

The listing is the **package page**, URL shape
`https://melpa.org/#/<package>` — a single-page app with hash routing
confirmed in the site source [5]. The page fetches and renders:

- the package name, version, description (the one-line `;;; … ---` header
  description from archive.json), dependencies, downloads, "Needed by"
  (reverse dependencies), and the package's **keywords** from the
  `Keyword:` header [5][8];
- a **Description section that is the package's readme.txt, fetched from
  `https://melpa.org/packages/<name>-readme.txt` and inserted into a `<pre>`
  element — plain text, never rendered markdown** [5];
- an automatically generated `https://melpa.org/packages/<name>-badge.svg`
  shield (the badge every package shows in its repo README) [5];
- a link to the *recipe* (`github.com/melpa/melpa/blob/master/recipes/<name>`)
  as the package's source of truth [5].

The readme.txt is produced at build time by `package-build--write-pkg-readme`,
which writes **the Commentary section of the main .el file** (`lm-commentary`)
— the repo's README.md is not consulted at all (grep package-build.el: the
only readme logic is the commentary path) [6]. So on MELPA, the listing copy
is literally `;;; Commentary:` in `epher.el`, shown as preformatted text.
No images, no markdown, no GIFs, no icon anywhere on the page.

## Publishing, step by step

### First publication (all human-only)

1. **Make the package repo-ready.** Requirements from CONTRIBUTING.org and
   the PR template, checked against `epher.el` [2][3]:
   - GPL-compatible license, **"preferably the GNU General Public License
     (GPL) version 3. The license boilerplate should be applied above the
     `;;; Commentary` of each source file"**, and a LICENSE/COPYING file in
     the repo. We have the MIT LICENSE file at the repo root, but
     `epher.el`'s header carries **no license boilerplate and no `Author:`
     header** — both need adding (archive.json exposes authors on every
     package page; ours would be empty) [2][13].
   - `Version:` header — present (`0.5.57`), kept in lockstep with the train
     [13]. (Unstable builds ignore it — see below — but stable and any
     human installing from source read it.)
   - `Package-Requires:` — present (`((emacs "29.1"))`); eglot is built in
     at 29, so no other deps [13].
   - Lexical binding cookie — present (`-*- lexical-binding: t; -*-`, a
     listed typical problem) [2][13].
   - Run `package-lint` (and `flycheck-package`/`checkdoc`) and fix what it
     flags; byte-compile cleanly; **do not** ship a README/changelog inside
     the package — files like those "would only end up in an obscure
     installation directory where a user would never know to look". Our
     single-file recipe ships exactly one file, so nothing else can sneak
     in [2].
   - If LLMs generated some of the code, an `Assisted-by:` line under the
     `Author:` line is requested ("the human author should be the first and
     most thorough reviewer") [2][3].
   - The package must have "been maintained in a public repository for 1
     month or more" (PR checklist) [3].
2. **Fork `melpa/melpa` and add `recipes/epher`**:

   ```elisp
   (epher
    :fetcher github
    :repo "upyesp/epher"
    :files ("clients/emacs/epher.el"))
   ```

   Recipe format per the README: `:fetcher github` + `:repo "user/repo"`
   (dedicated forge fetchers preferred over generic `git`); `:files` globs
   are relative to the repository root and "a file like `lisp/foo.el` would
   become `foo.el` in the new package", so the monorepo path is mechanics,
   not a blocker [1]. The recipe file name must equal the package name; no
   `epher-pkg.el` may be checked in (generated from our headers at build
   time) [1].
   One guideline to acknowledge honestly: CONTRIBUTING.org asks that each
   package live in a **dedicated SCM repository** ("This makes it possible
   to have a different version number for each package as MELPA stable looks
   at the SCM's tags"). Guidelines "are not strict and evaluated on a case
   by case basis given proper justification" [2]. Our justification is
   ADR-0066's version-locked monorepo; the `:files` spec and the shared
   `v0.5.x` tags make the stated concern moot (every client gets the same
   version number *by construction*). Expect a reviewer question; answer
   with the ADR.
3. **Test the recipe before the PR**, exactly as documented: `make
   recipes/epher` builds the package into `packages/`; `MELPA_CHANNEL=stable
   make recipes/epher` checks which version the tag machinery picks (this
   validates that our `v0.5.57` tag parses via the default
   `:version-regexp`, which matches `v4.3.5`-style tags); `make sandbox
   INSTALL=epher` gives a sandboxed Emacs with the new package installable
   [2].
4. **Open the PR** from a dedicated branch. Title must be **"Add recipe for
   epher"** (template instruction); fill the template: brief summary, direct
   repo link, your association (maintainer), "Relevant communications with
   the upstream package maintainer" → *None needed* for our own package,
   and the checklist (GPL-compatible ✓; package-lint ✓; byte-compiles ✓;
   checkdoc ✓; built and installed ✓; public repo ≥ 1 month) [3].
5. **CI runs on the PR itself** (ci.yml triggers on `pull_request`, builds
   the changed recipes) — "After submitting, fix any problems the CI
   reports" [3][4]. Then a maintainer reviews; expect "a week (sometimes
   several)" [2].
6. **After merge, everything is automatic.** The build server checks out the
   recipe's repo, builds `epher-<date>.el`/tar, generates
   `epher-readme.txt` from the Commentary, updates archive.json and the
   download counter. "Packages are updated at intervals throughout the
   day" [1]. No token, no secret, no account anywhere: the whole channel is
   GitHub PRs [2][3].

### Every later release train (nothing to do)

There is **no per-release action on MELPA, ever**. Unstable MELPA (the
default archive everyone uses) versions packages by the date of the last
commit that touched the recipe's files — live example: magit's version is
`20260926.1436` [8] — so any commit to `clients/emacs/epher.el` on main
becomes a new package build within a day, header `Version:` notwithstanding.
MELPA Stable versions from the repo's **tags**: "To have a stable version
generated for your package simply tag the SCM repository using a naming
compatible with the `version-to-list` function" [2] — our per-train
`v0.5.x` tags already do exactly that, so stable tracks `0.5.57`, `0.5.58`,
… automatically. One recipe feeds both archives; there is no separate
stable recipe format, only `MELPA_CHANNEL=stable` for local testing [2].
The maintainers themselves note they "do not use MELPA Stable themselves,
and do not particularly recommend its use" [1] — we submit the one recipe
and let stable ride the tags for free. Recipe edits are needed only if the
file path changes.

## Accounts, tokens, and what is automatable

| step | who | credential |
| --- | --- | --- |
| headers/license/package-lint on epher.el | human, once | — |
| fork + `recipes/epher` + PR | human, once | GitHub account |
| review responses | human, during review | GitHub account |
| build + publish after merge | MELPA's server | nothing of ours |
| per release train | **nobody** | nothing — main commits and tags drive it |

There is no account to create, no API key, no secret, no CI job. MELPA is
the only channel in this whole distribution effort where the repository
needs zero credentials. The flip side: nothing about it is CI-automatable
either — the PR and the review conversation are irreducibly human, and that
is by design.

## How the top five format their listings

From `download_counts.json` (2026-09-29) and each package's live
`-readme.txt` [7][9]:

| package | downloads | version | description (header line) | readme.txt | keywords |
| --- | --- | --- | --- | --- | --- |
| dash | 6,511,270 | 20260221.1346 | 31 chars — "A modern list library for Emacs" | 688 chars | extensions, lisp |
| s | 5,626,068 | 20260522.135 | 47 chars — "The long lost Emacs string manipulation library" | 113 chars | strings |
| magit | 5,309,084 | 20260926.1436 | 28 chars — "A Git porcelain inside Emacs" | 581 chars | git, tools, vc |
| f | 4,460,827 | 20241003.1131 | 49 chars — "Modern API for working with files and directories" | 125 chars | files, directories |
| helm | 3,900,751 | 20260926.1128 | 52 chars — "Helm is an Emacs incremental and narrowing framework" | 127 chars | 11 keywords |

Caveat on the ranking: dash, s and f are dependency *libraries* (they sit at
the top because every package.el install of anything depending on them
counts), not end-user applications; magit and helm are the flagship
end-user packages. For a *language-mode* comparison point: markdown-mode
sits just behind at 3,604,710 downloads [7].

Reading across them, the MELPA listing pattern:

- **The header description is one short sentence** (28–52 chars), naming
  what the thing is, no version, no markup. It appears in `list-packages`
  and on the page title — the single most-read line of the listing.
- **The readme.txt (= Commentary) is short plain text**: 113–688 chars
  across the top five — one-sentence positioning plus one to three short
  paragraphs. No markdown, no links, no bullets in four of five (magit's
  two paragraphs are prose).
- **No media exists anywhere on melpa.org** — the `<pre>` readme cannot
  carry images, and none of the top five try. Visuals live in the repos'
  GitHub READMEs, which MELPA never renders.
- **Keywords are real search terms** (git, strings, files…) set in the
  `Keyword:` header — helm carries eleven.

## Media spec: where the captures must go

MELPA's Description is `epher-readme.txt` — the Commentary of `epher.el` —
in a `<pre>` tag [5][6]. **Screenshots and GIFs will never render on
melpa.org**; a markdown image line would show as literal `![…](…)` text.
So: captures (the same demo GIF and hero screenshot the VS Code listing
uses, adapted) belong in `clients/emacs/README.md` and the repo's shared
images, which is what people following the `URL:` header link will see —
`clients/emacs/README.md` today has none [13][14]. The Commentary itself is
the listing copy: adapt it from `clients/vscode/README.md`'s intro and
"What you get" bullets, as plain text, no ADR numbers (the current first
line — "ADR-0066 ships Emacs as a ready-made config" — is internal jargon
sitting on the most-read line of the future listing [13]), no social links,
and remember every line gets `;;` stripped and lands preformatted.

## Icon

There is no icon on MELPA package pages: the only image-ish artifact is the
auto-generated `<name>-badge.svg` version shield [5], and NonGNU ELPA's
index is likewise text-only [10]. No package art is uploadable anywhere.
The epher monogram (`site/icon.svg`, with `icon-light.svg`/`icon-plain.svg`
variants) therefore still does its branding work only in the repo: the
README, and the **GitHub social preview** — which for upyesp/epher is
currently GitHub's auto-generated card, not the monogram [15]. Setting the
social preview (one-time, human) is what puts the monogram on the recipe
PR, the README links, and any shared URLs.

## NonGNU ELPA: the second queue, briefly

The process is an **email**, not a PR: "To submit a package to GNU or NonGNU
ELPA, all you have to do is to send an email to the emacs-devel@gnu.org
mailing list", with a brief description, a link to a public git repo, and
GNU-vs-NonGNU preference; sign up for the list first (or your address needs
manual approval); volunteers take it from there [12]. No copyright
assignment for NonGNU (that is the GNU ELPA criterion) [12]. The build
server pulls from the upstream repo, and the submission form explicitly
anticipates our layout — one of the "mention it in your message" items is
"Are the Lisp files in some other directory than the root directory of the
repository?" [12]. For the description, "we check if there is a README.org
file (including … other formats, but **excluding Markdown**). If none is
found … we will use the contents of the ;;; Commentary: section" — our
Markdown README is excluded by policy, so the Commentary serves here too
[12].

Is it worth it? It is the only channel that makes epher visible in
`M-x list-packages` **with no user configuration at all** (default archive
since Emacs 28) [11] — reach MELPA cannot match for an unconfigured Emacs.
The cost is a slow, human, mailing-list review with soft criteria (code
quality as an example for others, long-term maintenance commitment,
good-citizen behaviour) and a queue measured in emails, not CI minutes
[12]. Verdict: do it after MELPA settles, from the same `epher.el` and the
same Commentary — the marginal work is one email — but do not block the
MELPA PR on it.

## Gap analysis: epher.el today

Read against the MELPA requirements [2][3][13]:

Ready already:

- `Version: 0.5.57` in lockstep with the train; `Package-Requires
  ((emacs "29.1"))`; `Keywords: languages`; `URL:` to the repo.
- Lexical-binding cookie (a listed typical problem, already handled).
- Single-file package, no `-pkg.el` tracked, no README/changelog shipped in
  the package, no third-party code vendored.
- MIT LICENSE at the repo root (GPL-compatible ✓).
- Repo public for well over a month (the VS Code marketplace went live
  2026-09-18; the repo predates it).

To add or change:

1. **License boilerplate in the header** — the MIT notice lines above
   `;;; Commentary:`, per the guideline, plus an `Author:` header (the
   package page shows authors; ours would be blank).
2. **`Assisted-by:` line** if applicable, under `Author:` (PR checklist
   item) [3].
3. **Run package-lint + checkdoc + byte-compile** and fix findings before
   the PR; `make recipes/epher` and `MELPA_CHANNEL=stable make
   recipes/epher` locally [2].
4. **Rewrite the Commentary** as the listing copy: adapted from
   `clients/vscode/README.md`, no ADR numbers, lead with what the user
   gets (inline answers, diagnostics, hover, completion), mention the
   server binary download from the releases page. Plain text only — this
   is what melpa.org shows in the `<pre>` [5][6].
5. **Captures for `clients/emacs/README.md`** — the channel's visuals live
   there, not on melpa.org [13][14].
6. **Repo topics**: `neovim` for the rock channel; nothing MELPA-side needs
   topics, but keep the social preview/monogram consistent [15].

And the standing rules: listing text adapts from `clients/vscode/README.md`;
no social media links anywhere; the monogram is the brand; human-only steps
(license headers, recipe, PR, email) stay clearly separate from CI (which
does nothing on this channel, and that is a feature).

## Sources

All fetched 2026-09-29 unless noted.

1. MELPA README (usage, stable vs unstable and the maintainers' own
   assessment, recipe format, `:files` semantics, pkg.el rule, build
   scripts, version %Y%m%d): https://github.com/melpa/melpa/blob/master/README.md
2. MELPA CONTRIBUTING.org (guidelines, license boilerplate, package-lint,
   dedicated-repo and case-by-case clauses, tag-for-stable, test commands,
   PR flow, review timing): https://github.com/melpa/melpa/blob/master/CONTRIBUTING.org
3. MELPA pull-request template ("Add recipe for …", 1-month-public,
   package-lint/checkdoc/byte-compile checklist, Assisted-by): https://github.com/melpa/melpa/blob/master/.github/PULL_REQUEST_TEMPLATE.md
4. MELPA CI workflow (runs on push and pull_request): https://github.com/melpa/melpa/blob/master/.github/workflows/ci.yml
5. melpa.org site source (`readmeURL` `/packages/<name>-readme.txt`,
   rendered with `m("pre", …)`, badge SVG, hash routing, recipe link):
   https://github.com/melpa/melpa/blob/master/html/js/melpa.js
6. package-build (readme.txt written from `lm-commentary` of the main
   file, no README.md path): https://github.com/melpa/package-build/blob/master/package-build.el
7. Live download counts (7,259 names): https://melpa.org/download_counts.json
8. Live archive metadata (6,325 packages; versions, descriptions, keywords,
   authors): https://melpa.org/archive.json
9. Rendered listing text of the top five:
   https://melpa.org/packages/dash-readme.txt, …/s-readme.txt,
   …/magit-readme.txt, …/f-readme.txt, …/helm-readme.txt
10. NonGNU ELPA package index: https://elpa.nongnu.org/nongnu/
11. NonGNU ELPA about ("one of the two default package archives", default
    since Emacs 28.1, GNU-vs-NonGNU difference negligible for users):
    https://elpa.nongnu.org/about.html (the Contribute! link redirects to
    https://elpa.gnu.org/contributing.html)
12. ELPA contributing page (email-to-emacs-devel submission, FSF
    copyright assignment for GNU ELPA only, GPL-compatible license,
    monorepo question, README.org-not-Markdown description rule, soft
    criteria): https://elpa.gnu.org/contributing.html
13. Our file and its README: `clients/emacs/epher.el`,
    `clients/emacs/README.md` in this repo
14. Capture inventory: `clients/vscode/images/` (editor.png, demo.gif,
    hover.png, completion.png) vs `clients/emacs/` (no images)
15. GitHub social preview check: https://github.com/upyesp/epher (og:image
    = auto-generated `opengraph.githubassets.com` card, no custom preview
    set)
