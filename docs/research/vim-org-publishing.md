# Publishing epher for Vim on vim.org: the script page, the upload, and the human-only train

Research of 2026-09-29, in preparation for publishing the Vim client
(`clients/vim`: ftdetect, syntax, ftplugin — no LSP wiring in Vim itself) on
the official Vim Scripts index: www.vim.org/scripts. Four questions: what the
channel is and who still reads it, how a script is submitted and updated
today, what the page carries (and what it cannot), and what the most-rated
scripts do that ours should.

## Goal

Decide what it takes to put epher on vim.org with a script page that matches
what the top-rated scripts ship: the account work, the submission and
version-upload flow, the description conventions, and the honest shape of a
channel with no API, no icons, and no images — a text page, a file, and a
download counter.

## Method (all claims verified)

Everything below was fetched directly with curl on 2026-09-29, no blog posts:

- **vim.org itself**: the scripts index, the account registration page, the
  add-script flow (anonymous probe), the site help page (`huh.php`), the
  script pages of the five top-rated scripts plus htmldjango omnicomplete
  and ctrlp.vim, and the search results sorted by rating and by downloads
  [1]–[10].
- **The ratings/download columns** come from the site's own sortable search
  (`script_search_results.php?order_by=rating|downloads`) [7][8].
- **Our own artifacts**: `clients/vim/README.md`, the three runtime files,
  the release workflow, and `clients/vscode/README.md` (repo files).

## The channel

**Vim Scripts** at `https://www.vim.org/scripts/` is the official script
index on the Vim site itself ("vim online"). URL shapes:

- Scripts home: `https://www.vim.org/scripts/index.php` [1]
- Script page: `https://www.vim.org/scripts/script.php?script_id=<id>`
  (e.g. taglist is `script_id=273`) [4]
- Per-release download:
  `https://www.vim.org/scripts/download_script.php?src_id=<release id>` —
  the src_id is per version, not per script [4]
- Search/sort: `https://www.vim.org/scripts/script_search_results.php`
  with `query`, `script_type`, `order_by` (Rating | Downloads | Script
  Name | Creation Date), `direction` [7][8]

The channel is old but not dead: the top script has 363,303 downloads,
and the "Recent Script Updates" panel on the index shows a steady pulse —
the newest seven uploads span 2026-08-29 to 2026-09-20 [1][4]. The
audience is Vim (not Neovim)
users who install by downloading a package onto their runtimepath — exactly
the install story `clients/vim/README.md` documents. Neovim users get the
LSP wiring from `clients/nvim`; the vim.org listing covers the shared
runtime and the vim-lsp path.

## First publication, step by step

### The human-only steps (everything, on this channel)

1. **vim.org account.** Register at
   `https://www.vim.org/account/register.php`: user name, first name, last
   name, email, password, confirm password, and a copy-paste human check
   [2]. The page's own instruction sets the tone: "Please create an account
   only when you have a script tested and ready to upload" [2]. Accounts
   exist "so there can be defined contact people for scripts" [3].
2. **Submit the script.** Logged in, the scripts index offers "( add
   script )" → `https://www.vim.org/scripts/add_script.php` [1]. Anonymous
   requests are bounced to `login.php?referrer=%2Fscripts%2Fadd_script.php`
   (probed today) [5], so the form itself is visible only to an account holder.
   Its field inventory is the data model every script page displays [4]:
   - script **name**;
   - **summary** — the one-liner shown in the recent-updates list
     (`name : summary`);
   - **description** — the main text block (plain, preformatted text;
     below for what survives);
   - **install details** — a second text block ("How to install", rendered
     after the description);
   - **script type** — one of: color scheme, ftplugin, game, indent,
     syntax, utility, patch (the search facet's own list) [7]. For epher's
     client, `utility` (or `syntax` if we ship the syntax file alone);
   - the **first version**: package file upload, version string, Vim
     version compatibility, release notes [4].
3. **No approval queue is documented.** Nothing on the submission path, the
   registration page, or the site help mentions moderation or review — the
   contrast with Eclipse Marketplace's explicit moderation queue is total
   [1][2][3]. New uploads surface in the index's Recent Script Updates with
   their upload dates, which is the publication event [1]. Treat
   "submit, then check the index next day" as the verification step.
4. **No API exists.** `api.php` and `scripts/api.php` are 404 (probed
   today) [9]; the site is PHP web forms end to end, with no tokens, no
   OAuth, nothing a CI secret could hold. **Every step — submission, version
   upload, description edits — is a human at a browser.**

### What survives in a description

Evidence from the fetched pages:

- Descriptions are **preformatted plain text**; whitespace is preserved and
  long lines are not wrapped (taglist's description keeps its hand-set URL
  indentation) [4].
- **Bare URLs become links** — taglist's description shows
  `https://github.com/yegappan/taglist` as a real `<a href>` [4].
- **HTML tags pass through** — vim-htmldjango_omnicomplete's page carries
  literal `<br>` tags in the description markup, and its screenshot links
  are bare URLs out to GitHub (`.../examples/block_eample.png`), not
  embedded images [10]. Among the top five, though, the style is plain
  text with bare URLs; no script in the top five uses markup beyond that
  [4].
- **No images**: zero `<img>` in the descriptions of all five top scripts
  and in htmldjango omnicomplete. There is no screenshot field, no gallery,
  no file storage for media on script pages [4][10].
- **Upload sizes are capped** by the site, tightly enough that maintainers
  split archives around it — the hexpair release notes (index, fetched
  today) describe shipping a `.minimal.tar.bz2` "because vim.org/scripts
  has strict limits for upload size" [1]. Keep `epher-vim.zip` tiny (three
  text files today) and this is never a problem.

## What the top five do

From `script_search_results.php?order_by=rating&direction=descending`
(the site's own ranking), then each script page [7][4]. The "rating" column
is rating points, almost certainly on a 3-point-per-vote scale — the vote
buttons on every script page are Life Changing / Helpful / Unfulfilling,
and the top scripts' averages land near 2.9, which only 3/2/1 produces
(taglist: 14,532 points over 4,944 votes ≈ 2.94) [4]:

| script | rating (points / votes) | downloads | description | links in desc. | images |
| --- | --- | --- | --- | --- | --- |
| taglist.vim (273) | 14,532 / 4,944 | 363,303 | 815 chars | 3 | 0 |
| The NERD tree (1658) | 9,200 / 2,972 | 269,498 | 2,714 chars | 2 | 0 |
| snipMate (2540) | 6,534 / 1,957 | 104,227 | 1,552 chars | 2 | 0 |
| rails.vim (1567) | 6,437 / 2,239 | 107,957 | 2,496 chars | 3 | 0 |
| vimwiki (2226) | 6,313 / 1,663 | 37,780 | 3,278 chars | 1 | 0 |

Reading across them:

- **The first line is a plain declarative "X is ..."**: "The "Tag List"
  plugin is a source code browser plugin for Vim and provides an overview
  of the structure of source code files...", "A tree explorer plugin for
  navigating the filesystem" [4][7].
- **Descriptions are 800–3,300 characters of plain text**: what it does,
  where the repository and docs live (bare URLs), how to get help
  (`:help taglist`). That is the whole format — no badges, no banners, no
  feature bullets with screenshots [4].
- **Install details is its own block**, numbered steps, runtimepath
  (`$HOME/.vim`, `$HOME/vimfiles`, `$VIM/vimfiles`) plus `:helptags` when
  there is help [4].
- **Every page ends in the versions table**: package | script version |
  date | Vim version | user | release notes — the release notes are where
  changelogs live [4].
- Sorted by downloads instead of rating, the same names lead (taglist,
  NERD tree, then c.vim) — rating and downloads agree on what the canon is
  [8].

## Media and icon spec

- **There is no icon field, no logo, no avatar** on script pages — the page
  is name, type, dates, description, install details, ratings, versions
  [4]. The epher monogram (`site/icon.svg`) has nowhere to go on vim.org;
  it stays the brand of the GitHub release artifacts and the website, and
  the script page identifies epher by name and prose only.
- **No images anywhere on the page**: the description field is text, and
  none of the top scripts carry any `<img>` [4][10]. If the listing should
  show the inline-answers demo, the description links out to it
  (epher.org or the GitHub raw URL) rather than embedding.
- **One file per release**, the package upload (zip expected; taglist ships
  `taglist_46.zip`, NERD tree `NERD_tree.zip`) [4]. Our `epher-vim.zip` —
  ftdetect, syntax, ftplugin — is exactly the expected shape: download,
  extract onto the runtimepath.

## Listing copy source

The listing text adapts from `clients/vscode/README.md` (repo file):
the "epher is a calculator language" intro and the feature story, compressed
to the plain-text conventions above. Adaptations for Vim: the install
section is the runtimepath story from `clients/vim/README.md` (copy the
directory, or extract `epher-vim.zip` into `~/.vim`), the vim-lsp wiring
snippet is the inline-answers path, and the Neovim pointer says the native
LSP glue lives in `clients/nvim`. Links out: github.com/upyesp/epher,
epher.org, the releases page. **No social media links.**

## Human vs. CI, across the release train

One version across all clients per train (extension 0.5.x speaks epher
0.5.x).

- **CI (automatable)**: build `epher-vim.zip` (already does — ftdetect,
  syntax, ftplugin from `clients/vim`) and attach it to the GitHub release
  with the stable name. The zip doubles as the vim.org package upload; the
  GitHub release carries everything else (`epher-lsp` binaries the
  vim-lsp setup points at).
- **Human (one-time)**: create the vim.org account (only once the script is
  ready, per the site's own advice), submit the script through
  `add_script.php`.
- **Human (per train, small)**: "upload new version" on the script page →
  `add_script_version.php?script_id=<id>` [6] — new package file, version
  string, Vim version, release notes [4]. There is no way for CI to do
  this; a release checklist line, like the Eclipse listing edit.
- **CI secrets**: none possible — vim.org has no API and no tokens [9].

## Sources

All fetched 2026-09-29 unless marked as repo files.

1. Vim Scripts index (add-script link, recent updates, upload-size remark in hexpair's notes): https://www.vim.org/scripts/index.php
2. Account registration (fields, "tested and ready to upload"): https://www.vim.org/account/register.php
3. Site help, "Scripts" (account required to upload; contact-person rationale): https://www.vim.org/huh.php
4. Script pages of the top five (page anatomy: description, install details, rate buttons, versions table, upload-new-version link): https://www.vim.org/scripts/script.php?script_id=273 (taglist), 1658 (The NERD tree), 2540 (snipMate), 1567 (rails.vim), 2226 (vimwiki)
5. Add-script flow, anonymous probe (redirect to login): https://www.vim.org/scripts/add_script.php → https://www.vim.org/login.php?referrer=%2Fscripts%2Fadd_script.php
6. Version-upload URL shape (link on script pages): https://www.vim.org/scripts/add_script_version.php?script_id=273
7. Search sorted by rating (top table; script-type and order options): https://www.vim.org/scripts/script_search_results.php?order_by=rating&direction=descending
8. Search sorted by downloads: https://www.vim.org/scripts/script_search_results.php?order_by=downloads&direction=descending
9. No API: https://www.vim.org/api.php and https://www.vim.org/scripts/api.php (both 404)
10. htmldjango omnicomplete page (HTML `<br>` in description; screenshot as a bare linked URL, no images): https://www.vim.org/scripts/script.php?script_id=4027; ctrlp.vim for comparison: https://www.vim.org/scripts/script.php?script_id=3736

## Addendum 2026-10-05: the per-train upload is CI-driven now

The verdict above ("every step is a human at a browser", "CI secrets:
none possible") described the channel as it advertises itself: no API,
no tokens. It held until the requirement changed: with an explicit ask
to automate the version upload, credentials held as CI secrets, the
plain PHP forms turn out to be drivable end to end without a browser:

- Login is a plain POST to `/login.php` (`authenticate=true`,
  `userName`, `password`); success is visible as a session cookie,
  while a failed login re-renders the form and sets no cookie. No CSRF
  token, no captcha on login — the registration page's human check is
  registration-only (probed 2026-10-05 with a bogus-credentials POST:
  HTTP 200, no Set-Cookie, form re-rendered).
- The version-upload form (`add_script_version.php?script_id=<id>`) is
  a stock multipart form, reachable only with a session.

What remains genuinely true, and is the honest cost of this automation:
this is form-driving, not an API. It can break whenever the site
changes its forms (the uploader therefore parses the form at runtime
and echoes its fields back, overriding only the release version, the
Vim version, the release notes, and the package file); and the CI
secret is a full account password, a weaker credential than every
other channel's scoped token. vim.org offers nothing stronger.

The form-driving bit back once, instructively: the version form carries
two same-named submit buttons (`add_script=upload`,
`add_script=cancel`); echoing every field sent `cancel` last, and
PHP's last-value-wins made every POST cancel itself — the server
greeted each attempt with a friendly redirect and saved nothing. The
uploader now sends exactly one submit, the way a browser's clicked
button does. The catch-up upload of the pending 0.5.59 lean zip landed
on 2026-10-05 (run 37377102581, verified in the versions table).

Implementation: `clients/vim/listing/publish-vimorg.py` (stdlib only:
login, form echo, multipart POST, versions-table verification,
`--dry-run`) called by `.github/workflows/vim-publish.yml` on `v*`
tags; credentials live in the stores environment as
`VIMORG_USERNAME` / `VIMORG_PASSWORD`.
