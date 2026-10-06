# Publishing epher for Neovim: luarocks.org, awesome-neovim, and what our config layout can ship as

Research of 2026-09-29, in preparation for opening a distribution channel for
the Neovim client. Four questions: what counts as the "official" Neovim
channel when there is no first-party store, what a listing there looks like,
what publishing actually requires of a human and of CI (and which of our
layout constraints bite), and what the top rocks and list entries do that we
do not. As with the other publishing research, this document prices the
channel; it does not unpark anything (ADR-0066 still ships Neovim as a
ready-made config, not a plugin).

## Goal

Decide what it takes to make `epher` installable the way Neovim users
actually install things — `:Rocks install epher` for the rocks.nvim crowd,
an rtp path for everybody else — and get the client onto the discovery lists,
with a listing whose text adapts from `clients/vscode/README.md`.

## Method (all claims verified)

Everything below was fetched directly with curl, no blog posts:

- **Neovim's own docs**: the Lua guide and user manual chapter 5 (current
  release docs and `master` vimdoc), the 0.10, 0.11 and 0.12 news files, and
  the neovim.io homepage [1][2][3][4][5].
- **The rocks.nvim ecosystem**: the rocks.nvim README and `:help rocks`
  (the project now lives at `lumen-oss`, both `mrcjkb/rocks.nvim` and
  `nvim-neorocks/rocks.nvim` 301-redirect there) [6][7][8].
- **LuaRocks**: luarocks.org pages (home, about, the `neovim` label, the
  rock pages of the top nvim rocks), the official docs in the luarocks repo
  (`creating_a_rock`, `luarocks_upload`, `luarocks_new_version`,
  `rockspec_format`, `namespaces`, the Makefile guide), the September 2026
  security-incident report, and a live rockspec download [9]-[19].
- **The CI action**: `mrcjkb/luarocks-tag-release` (redirects to
  `lumen-oss/luarocks-tag-release`), its `action.yml` and README at `master`,
  plus the sample plugin repo it links [15][16][20].
- **awesome-neovim**: the README (1,484 entries), CONTRIBUTING.md,
  MAINTAINERS.md and the PR template of `rockerBOO/awesome-neovim`
  (21,426 stars), and neovimcraft.com, the site neovim.io's "Get Plugins"
  button points at [21]-[24][5].
- **Popularity data**: luarocks label pages with per-rock download counts,
  GitHub star counts read off the repo pages, and the top repos' READMEs
  checked for embedded media [12][25][26].

## The reality check: there is no store, but there is a registry and a list

Neovim has no first-party marketplace, and its own docs bless none. The user
manual says only: "You can find packages on the Internet in various places"
[2]; the Lua guide has no "where to find plugins" section at all [1]; the
0.10 and 0.11 release notes mention no registry or store [3]. What exists:

- **luarocks.org is the registry the ecosystem converged on.** The
  rocks.nvim package manager (`:Rocks install …`) installs directly from it
  [6]; its README points plugin authors at the `neovim` label page
  (`https://luarocks.org/labels/neovim`) as the place plugins gather [6]; and
  the Neovim org itself publishes `nvim-lspconfig` there, under the uploader
  account `neovim`, currently at 70,615 downloads [14]. That last fact is the
  strongest "official channel" signal: the core team's own plugin ships as a
  rock. It is still a community-run site ("a community run rock and rockspec
  hosting service for Lua modules. Anyone can join and upload" [10]), not
  something Neovim the project operates.
- **awesome-neovim is the discovery list.** 21,426 stars, 1,484 entries,
  PR-curated [21][22]. neovimcraft.com — where neovim.io's own **Get
  Plugins** button on the homepage points [5] — is a search front-end whose
  data is "scraped from github" off that list; it even shows the same
  categories (its "programming-languages-support" category, where epher
  belongs, holds 107 entries) [23][24]. Land on awesome-neovim and you are
  on neovimcraft shortly after.
- **vim.org and vim-awesome are the Vim-era legacy**: vim.org still hosts
  scripts, and vim-awesome describes itself as a directory for Vim plugins
  [27]. Neither is where Neovim users look today.

Note for 0.9 users: rocks.nvim requires Neovim >= 0.10 [6], and 0.12 adds a
built-in plugin manager (`vim.pack`) that installs from git URLs, not a
registry [4]. So the rock reaches the 0.10/0.11 rocks.nvim crowd; the manual
rtp path in `clients/nvim/README.md` stays the story for everyone else —
publishing a rock changes the channel, not the config.

## What a listing is on luarocks.org

The listing is the **rock page**, URL shape
`https://luarocks.org/modules/<user>/<package>` (e.g.
`/modules/neorocks/rocks.nvim`) [13]. It renders exactly:

- the package name, Follow/Star links, and the `luarocks install <name>`
  command;
- **`description.summary`** (one line, next to the name) and
  **`description.detailed`** (a paragraph under it) — rendered as plain text
  with line breaks, not markdown: bullets show as literal `-` dashes, URLs
  as bare text [13]. The site source has no markdown renderer [11].
- Uploader (with gravatar avatar), License (SPDX string from the rockspec),
  Homepage link, total Downloads;
- the version list — every `<version>-<specrev>` ever uploaded, each with
  its own age and per-version download count [13];
- Dependencies, "Dependency for" (reverse deps), Labels (free-form tags;
  the `neovim` label is how you land on the label page), Manifests [13].

The underlying files live at
`https://luarocks.org/manifests/<user>/<name>-<version>-<specrev>.rockspec`
(the rockspec is public and fetchable [12]) and per-user manifests at
`https://luarocks.org/manifests/<user>`; the root manifest is global and
first-come first-served on package names [18]. **No README is shown, no
images, no screenshots, no icon field** — the rockspec format has no such
fields at all [17]; the only picture on a rock page is the uploader's
gravatar [13].

## Publishing, step by step

### Human-only steps (one-time)

1. **Create a luarocks.org account** — register directly or via GitHub
   login [20].
2. **Generate an API key** at `https://luarocks.org/settings/api-keys`
   (Settings → API keys → Generate New Key) [15][20]. Name the repository
   secret `LUAROCKS_API_KEY` — that is the variable name the official
   action reads [15].
   **Important, dated fact: luarocks.org had a security incident in
   September 2026** (RCE, exploited July–August 2026, disclosed 2026-09-25).
   Every API key on the server was revoked and every session ended; the
   report's instruction is literally "Create a new API key" [19]. So the key
   we store must be minted after 2026-09-26, and any key older than that is
   dead.
3. **Decide the rock layout** (below) — this is a repo change, made once.
4. **File the awesome-neovim PR** (below) — human-only, GitHub account
   suffices.
5. **Set the GitHub social preview** (see Icon) — human-only, one-time.

### CI-automatable steps (per release)

6. **The tag triggers the upload.** The official action is
   `luarocks-tag-release` (authored by MrcJkb; `mrcjkb/luarocks-tag-release`
   301-redirects to `lumen-oss/luarocks-tag-release`; pin `@v7`) [15][16].
   Its documented shape:

   ```yaml
   name: LuaRocks release
   on:
     push:
       tags: ["*"]
     pull_request: # tests a local install, uploads nothing
   jobs:
     luarocks-release:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - uses: lumen-oss/luarocks-tag-release@v7
           env:
             LUAROCKS_API_KEY: ${{ secrets.LUAROCKS_API_KEY }}
   ```

   (the README's own example, with the action ref updated to the
   post-redirect org) [15]. It generates a rockspec from repository
   metadata (no rockspec committed per release), tests a local install
   from it, uploads, then tests the installed-from-server rock [15]. On
   `pull_request` it runs the whole install test without uploading — free
   CI for the packaging [15].

7. **Tag → version mapping**: the action defaults `version` to
   `github.ref_name`; a leading `v` is stripped, so tag `v0.5.57` becomes
   rock version `0.5.57-1` (`modrev` = `0.5.57`, `specrev` = `1`; see the
   live `rocks.nvim-2.49.0-1.rockspec` for exactly this shape, `git_ref =
   'v2.49.0'`, `source.url` = the GitHub archive zip of the tag) [15][12].
   This matches our release train one-to-one: one `v0.5.x` tag per train, one
   rock per train, version-locked like every other client (ADR-0066).

8. **Same-version republish**: luarocks rejects re-uploading a version that
   already exists; the CLI's `--force` exists precisely to overwrite "if the
   same version of the package already exists" [14]. The action is softer:
   by default it *does nothing* when the version is already published, and
   only fails when `fail_on_duplicate: true` [15]. Since we never re-push a
   train tag, this never bites; keep the default so a re-run of a release
   job stays green.

9. **Action inputs that matter for us** [15]:
   - `name`: defaults to the repo name (`epher`) — correct already.
   - `copy_directories`: defaults to `{{ neovim.plugin.dirs }}`, which
     expands to Neovim's runtimepath directories — autoload, colors,
     compiler, doc, filetype.lua, ftdetect, ftplugin, indent, keymap, lang,
     lsp, menu.vim, parser, plugin, queries, query, rplugin, spell, syntax —
     "only directories that exist will be copied" [15]. Our `ftdetect/`,
     `ftplugin/`, `syntax/` are all in the default set.
   - `labels`: defaults to the repo's GitHub topics — make sure `neovim` is
     a repo topic, or pass it explicitly; the label is what puts us on
     `luarocks.org/labels/neovim` [15][6].
   - `summary`/`detailed_description`: default to the GitHub repo about
     text; set them explicitly from the adapted listing copy.
   - `license`: fetched from GitHub's license detection (MIT → fine).
   - Limits, verbatim from its README: works only on **public** repos, and
     it installs with lua 5.1 (fine — the client is pure Lua) [15].

### Can our layout ship as a rock? (concrete)

A rock installs into the luarocks tree
(`…/lib/luarocks/rocks-5.1/<name>/<version>/`); `lua/` content lands on
`package.path` via `share/lua/5.1`, and rocks.nvim adds the *rock prefix
directory itself* to the runtimepath (`dynamic_rtp`, default on) so
`copy_directories` content — `ftdetect/`, `ftplugin/`, `syntax/`, `doc/`,
`plugin/` — just works, with `rtp.nvim` (a rocks.nvim dependency) sourcing
the ftdetect/plugin dirs [6][7][15]. That is the entire rocks.nvim plugin
convention: a normal plugin layout, packaged by the action's defaults.

Our layout today: `clients/nvim/lua/epher.lua` (the module) plus the
runtime files one level over in `clients/vim/{ftdetect,ftplugin,syntax}`
[28]. The action's default template installs from the **repository root**;
`copy_directories` takes flat directory names at the source root only
[17]. So the default recipe would find no `lua/` and no runtime dirs — the
monorepo is the one real obstacle. Three concrete shapes:

1. **Sync the files into `clients/nvim/` (recommended).** Give
   `clients/nvim/` the standard plugin layout — `lua/epher.lua`,
   `ftdetect/epher.vim`, `ftplugin/epher.vim`, `syntax/epher.vim` — with the
   three runtime copies sync'd from `clients/vim/` by the build, exactly the
   pattern the VS Code client already uses for `clients/shared` ("the
   grammar and snippets are copies … edit there, never the copies" [29]).
   Then the action's pure defaults work: `lua/` by the builtin convention,
   the three runtime dirs via the `{{ neovim.plugin.dirs }}` default. No
   custom rockspec, no Makefile.
2. **Custom template + Makefile.** Keep the current paths, pass `template:`
   a rockspec with `build.type = "make"` whose Makefile copies
   `clients/nvim/lua/epher.lua` into `$(INST_LUADIR)` and stages
   `clients/vim/*` at the source root where `copy_directories` finds them.
   The make backend passes `PREFIX`/`BINDIR`/`LIBDIR`/`LUADIR`/`CONFDIR`
   through `install_variables` for exactly this [14]. Works, but it is
   moving parts for nothing; only worth it if the copies in option 1 are
   unacceptable.
3. **Separate plugin repo.** What most rocks do, and what ADR-0066
   explicitly declined ("clients live in this repository, version-locked to
   epher's 0.5.x train") [30]. Not on the table unless the user reopens it.

Note one UX nicety either way: the rock delivers the module and the runtime
files, but `require("epher").setup(...)` is still the user's call — a rock
does not run setup. If we ever want zero-config, a tiny `plugin/epher.lua`
in the rock (in the default `copy_directories` set) could call setup with
defaults, and a `lsp/epher.lua` file would serve 0.11+ `vim.lsp.enable`
users; `lsp` and `plugin` are both in the action's default copy list [15].
That is a product decision, not a publishing one.

### awesome-neovim: the PR mechanics

- Fork `rockerBOO/awesome-neovim`, add one line to the README section that
  fits — for us **Programming Languages Support** (38 entries today;
  epher is language support in exactly this list's sense) [21][22].
- The entry format is `- [user/repo](URL) - Description.` — description
  ends with a period, no emojis, and **must not contain the words "plugin"
  or "Neovim"** ("it's obvious from the rest of the document"); spell
  Neovim/Vim/Lua/Tree-sitter capitalized when unavoidable [22][24].
- PR rules: title must match `Add|Update|Remove \`username/repo\`` (backticks
  included), **one plugin per PR**, use the PR template checklist (licensed;
  line ends with `.`; no "… for Neovim"; no emojis; acronyms capitalized)
  [22][24].
- Acceptance criteria (MAINTAINERS.md): must be Neovim-specific, functional,
  open-source licensed, quality README with install/usage, actively
  maintained, and **at least a week old** — younger entries get the
  `pending-merge` label [24].
- Run `./scripts/readme-check.sh` locally before pushing [22].
- After merge: nothing. The entry is static text; it never needs updating on
  a release train. neovimcraft picks it up via its scrape [23].

Our entry would read, adapted from the VS Code listing copy [29] and obeying
the rules:

```
- [upyesp/epher](https://github.com/upyesp/epher) - Calculator language with
  answers inline, LSP server glue, and syntax for `.epher` scripts.
```

## Accounts, tokens, and what is automatable

| step | who | credential |
| --- | --- | --- |
| luarocks.org account | human, once | GitHub login suffices [20] |
| API key `LUAROCKS_API_KEY` | human, once (post-incident minting) | repo secret; read only by the release job [15][19] |
| rock upload per train | CI (tag push) | `luarocks-tag-release@v7` + the secret [15] |
| rockspec/rockspec-template maintenance | human, when the layout changes | — |
| awesome-neovim PR | human, once | GitHub account [22] |
| neovimcraft | nothing | automatic scrape [23] |

Nothing else in the channel has or needs a token. No account exists or is
needed for awesome-neovim beyond GitHub itself; MELPA-style email flows do
not exist here.

## How the top five format their listings

From the live `neovim` label pages (luarocks downloads) and the repos
themselves [12][13][26]:

| rock | downloads | summary (chars) | detailed (chars) | license | labels | media on rock page |
| --- | --- | --- | --- | --- | --- | --- |
| plenary.nvim | 607,143 | 37 — "lua functions you don't want to write" | 106 | MIT/X11 | neovim | none |
| rest.nvim | 308,599 | 40 — "A fast Neovim http client written in Lua" | 128 | GPL-3.0 | neovim, rest | none |
| fidget.nvim | 299,120 | 65 | 493 (plain-text bullet list) | MIT | neovim, vim | none |
| nvim-web-devicons | 192,772 | 26 — "Nerd Font icons for neovim" | 104 (ends in a bare URL) | MIT | neovim | none |
| telescope.nvim | 152,809 | 51 — "Find, Filter, Preview, Pick. All lua, all the time." | 173 | MIT | neovim | none |

Reading across them, plus `nvim-lspconfig` (70,615 downloads, uploader
`neovim` itself, labels `neovim, lsp`, summary is one 83-char sentence)
[14]:

- **Summaries are one short sentence** (26–83 chars), no version numbers, no
  badges. The detailed text is one short paragraph (100–500 chars), often
  just the repo README's intro re-pasted.
- **Every one carries the `neovim` label** — that is what lists it on the
  label page; a rock without it is invisible there [6][12].
- **The rock page carries no media at all**, so the READMEs do the selling:
  plenary 1 image, rest.nvim 6 images, fidget 3 images + 1 GIF, telescope
  3 images + 1 GIF, nvim-web-devicons 0 [26].
- On the discovery-list side, the flagships (stars read off GitHub today)
  are lazy.nvim 21,609 / telescope.nvim 19,803 / nvim-treesitter 14,433 /
  nvim-lspconfig 13,960 / mason.nvim 10,496 — all with screenshots or a demo
  GIF embedded in the repo README (mason: 5 images; lazy: 1; telescope:
  3 + GIF; nvim-treesitter: 1) [25][26]. (nvim-treesitter itself is *not* on
  luarocks — only satellite rocks like `nvim-treesitter-context`, published
  by the neorocks account; parsers ride rocks-treesitter/NURR instead [6].)

The shared pattern: one-line summary, short plain-text detailed paragraph,
`neovim` label, permissive license, and a repo README that carries all the
visuals the registry cannot.

## Media spec: where the captures must go

The rock page renders only `summary` + `detailed` as text — no markdown, no
images, ever [13][17]. So the captures are a **repo-README asset**, not a
registry asset: they belong in `clients/nvim/README.md` (and shared repo
images), where awesome-neovim reviewers, neovimcraft visitors and anyone
following the rock page's Homepage link will see them. Today
`clients/nvim/README.md` has **zero images** while `clients/vscode/README.md`
has the hero screenshot, feature screenshots and a demo GIF [28][29]. If the
nvim channel opens, the captures we already produce for the other clients
adapt: hero (inline answers), a run-pane capture for `:EpherRun`, absolute
GitHub raw URLs like the vscode README uses. The `detailed_description` the
action uploads should be the 2–4 adapted sentences from the VS Code listing
intro, with no URL or emoji tricks — plain text is all it will ever be.

## Icon

There is no icon anywhere in the chain: the rockspec format has no image or
icon field [17], the rock page shows only the uploader's gravatar [13], and
the action has no input for one [15]. The epher monogram
(`site/icon.svg`, plus the `icon-light.svg` / `icon-plain.svg` variants)
stays the brand where it already works: the repo README, and the **GitHub
social preview** — which today is GitHub's auto-generated opengraph card for
upyesp/epher, not the monogram [31]. Setting the custom social preview
(repo Settings → Social preview) is the one human step that puts the
monogram on every awesome-neovim PR and rock-page Homepage link preview.

## Version and update mechanics across the release train

One version across all clients per train (ADR-0066) maps like this:

- **Tag `v0.5.x`** (the existing release-train act) → the `luarocks-tag-release`
  job uploads rock `0.5.x-1`. Nothing else changes: same tag, same job that
  already builds `epher-lsp-*` assets. One new CI job, one secret.
- **MELPA-equivalent "nothing to do"** does not exist here — luarocks is
  tag-pinned, unlike the Emacs channel; but the work is zero beyond having
  the workflow, because the version comes from the tag we already push.
- **awesome-neovim/neovimcraft**: nothing, ever, after the merge.
- Re-runs and hotfixes: a re-pushed train tag is not a thing in our process
  (the release tag is the last act of a promotion), so the same-version rule
  never triggers; if it ever does, the action no-ops rather than failing
  [15].

## Gap analysis: what we are missing today

1. **Rock layout**: `clients/nvim/` needs the plugin-dir shape (option 1
   above) before the action's defaults work; the runtime files are one
   directory over [28].
2. **No release workflow for luarocks**: the tag-triggered job does not
   exist yet; add `luarocks-tag-release@v7` with `LUAROCKS_API_KEY`, labels
   including `neovim`, and explicit `summary`/`detailed_description` [15].
3. **`neovim` topic** on the repo (the action derives labels from topics) [15].
4. **No captures in `clients/nvim/README.md`** — the one channel where our
   story (answers appearing inline as you type) is invisible without a GIF
   [28][29].
5. **No awesome-neovim entry** — and the week-old acceptance criterion is
   already met (the repo is months old), so the PR can go any time [22][24].
6. **Social preview** still the auto card; set the monogram [31].
7. **Not a gap but a rule**: the listing text (summary, detailed,
   awesome entry) adapts from `clients/vscode/README.md`; no social media
   links anywhere; the monogram is the brand [29].

## Sources

All fetched 2026-09-29 unless noted.

1. Neovim Lua guide (no plugin-store/installation section): https://neovim.io/doc/user/lua-guide/
2. Neovim user manual ch. 5, "Adding a package" (master): https://raw.githubusercontent.com/neovim/neovim/master/runtime/doc/usr_05.txt
3. Neovim 0.10 and 0.11 news (no registry mentions): https://neovim.io/doc/user/news-0.10/ and https://neovim.io/doc/user/news-0.11/
4. Neovim 0.12 news, "Built-in plugin manager: vim.pack": https://raw.githubusercontent.com/neovim/neovim/master/runtime/doc/news-0.12.txt
5. neovim.io homepage, "Get Plugins" → https://neovimcraft.com/: https://neovim.io/
6. rocks.nvim README (rocks.nvim 301-redirects to lumen-oss): https://github.com/lumen-oss/rocks.nvim (fetched via https://raw.githubusercontent.com/nvim-neorocks/rocks.nvim/main/README.md)
7. rocks.nvim `:help rocks` — `dynamic_rtp`, rtp.nvim: https://raw.githubusercontent.com/lumen-oss/rocks.nvim/master/doc/rocks.txt
8. Redirect check: https://github.com/nvim-neorocks/rocks.nvim → 301 https://github.com/lumen-oss/rocks.nvim; https://github.com/mrcjkb/luarocks-tag-release → 301 https://github.com/lumen-oss/luarocks-tag-release
9. luarocks.org home ("Most Downloaded", label index): https://luarocks.org/
10. luarocks.org about (community-run, manifests): https://luarocks.org/about
11. luarocks-site source (no markdown renderer): https://github.com/luarocks/luarocks-site
12. Live rockspec of rocks.nvim 2.49.0-1 (template shape, tag→version): https://luarocks.org/manifests/neorocks/rocks.nvim-2.49.0-1.rockspec
13. Rock page anatomy (rocks.nvim; also plenary/rest/fidget/devicons/telescope/lspconfig): https://luarocks.org/modules/neorocks/rocks.nvim
14. luarocks docs: upload (`--api-key`, `--force`, `--dry-run`): https://github.com/luarocks/luarocks/blob/main/docs/luarocks_upload.md ; new_version (tag `v` stripping): https://github.com/luarocks/luarocks/blob/main/docs/luarocks_new_version.md ; Makefile guide (PREFIX/LUADIR install_variables): https://github.com/luarocks/luarocks/blob/main/docs/creating_a_makefile_that_plays_nice_with_luarocks.md
15. luarocks-tag-release action.yml and README (`LUAROCKS_API_KEY`, `{{ neovim.plugin.dirs }}`, `fail_on_duplicate`, v-prefix, public-repo and lua-5.1 limits): https://github.com/lumen-oss/luarocks-tag-release (fetched at https://raw.githubusercontent.com/mrcjkb/luarocks-tag-release/master/action.yml and …/README.md)
16. Author attribution: Marc Jakobi in action.yml; the sample repo: https://github.com/lumen-oss/sample-luarocks-plugin (account + API-key walkthrough)
17. Rockspec format (build.install, copy_directories, no icon/image fields): https://github.com/luarocks/luarocks/blob/main/docs/rockspec_format.md
18. Namespaces and manifests: https://github.com/luarocks/luarocks/blob/main/docs/namespaces.md
19. LuaRocks.org Security Incident, September 2026 (all API keys revoked): https://luarocks.org/security-incident-september-2026
20. API keys page (linked from the action's prerequisites): https://luarocks.org/settings/api-keys
21. awesome-neovim README (1,484 `- [` entries; Programming Languages Support section; rocks.nvim entry): https://github.com/rockerBOO/awesome-neovim (21,426 stars)
22. awesome-neovim CONTRIBUTING.md (title regexp, one plugin per PR, no "plugin"/"Neovim", readme-check.sh): https://github.com/rockerBOO/awesome-neovim/blob/main/CONTRIBUTING.md
23. neovimcraft.com (scraped data, categories, "Submit a plugin"): https://neovimcraft.com/ and https://neovimcraft.com/about
24. awesome-neovim PR template and MAINTAINERS.md acceptance criteria (license, week-old, `pending-merge`): https://github.com/rockerBOO/awesome-neovim/blob/main/.github/pull_request_template.md and …/MAINTAINERS.md
25. Star counts read from repo pages (aria-label "… users starred this repository"): https://github.com/folke/lazy.nvim, https://github.com/nvim-telescope/telescope.nvim, https://github.com/nvim-treesitter/nvim-treesitter, https://github.com/neovim/nvim-lspconfig, https://github.com/williamboman/mason.nvim
26. Top-repo READMEs (embedded media counts): https://github.com/nvim-lua/plenary.nvim, https://github.com/NTBBloodbath/rest.nvim, https://github.com/j-hui/fidget.nvim, https://github.com/nvim-tree/nvim-web-devicons, https://github.com/nvim-telescope/telescope.nvim
27. Legacy Vim directories: https://github.com/vim-awesome/vim-awesome and https://www.vim.org/scripts/
28. Our clients: `clients/nvim/README.md`, `clients/nvim/lua/epher.lua`, `clients/vim/` (ftdetect/ftplugin/syntax) in this repo
29. Listing-copy source and sync precedent: `clients/vscode/README.md` ("the grammar and snippets are copies of clients/shared … edit there, never the copies")
30. ADR-0066 (ready-made configs, version-locked clients, not shipped plugins): `docs/adr/0066-ide-extensions-speak-one-lsp-server.md`
31. GitHub social preview check: https://github.com/upyesp/epher (og:image = auto-generated `opengraph.githubassets.com` card, no custom preview set)

---

## Addendum (2026-10-06): the first awesome-neovim rejection, and the mirror decision

### What happened

PR [rockerBOO/awesome-neovim#2532](https://github.com/rockerBOO/awesome-neovim/pull/2532)
("Add \`upyesp/epher\`", opened 2026-09-30) added epher to the
Programming Languages Support section with a fully checklist-compliant
line, and was closed the same day by maintainer DrKJeff16:

> The repository does not state Neovim support anywhere, rejecting.

The objection is factually correct: the PR linked the monorepo root
(`github.com/upyesp/epher`), and the root README contained no mention
of Neovim, nvim, or Vim (the Neovim client lives at `clients/nvim`,
one level down, invisible from the landing page). Every neighboring
entry in the section (`redpierrot/ballerina.nvim`,
`simonwinther/cppman.nvim`, `jgonmor16/hdlsnip.nvim`) links a
dedicated plugin repository named `*.nvim` whose landing page is a
Neovim plugin page. The list line itself passed every template lint;
the failure was the link target.

The PR was closed outright (not "changes requested"), so the
maintainers' one-week dispute window has no foothold: the etiquette
here is a fresh PR once the target repository is right.

### What the reviewers actually require (MAINTAINERS.md, acceptance criteria)

- Must be Neovim-specific (compatible and usable in Neovim).
- Must be functional and usable (proven-broken plugins are excluded).
- Must carry an open-source license (MIT or Apache 2.0 recommended
  where missing).
- Should have a quality README with sufficiently detailed
  installation/usage instructions.
- Should be actively maintained (recent commits).
- Should be at least a week old; younger plugins get the
  `pending-merge` label: approval now, merge after the week.

Etiquette (CONTRIBUTING.md + the PR template): title
`Add \`username/repo\``, one plugin per PR, no "plugin"/"Neovim" in
the description line, no emojis, `Neovim`/`Vim`/`Lua`/`Tree-sitter`
capitalized, acronyms uppercase, lines end with a period.

### How the five most popular plugins are built, and why they pass

| plugin | stars (2026-10-06) | list section | structure |
| --- | --- | --- | --- |
| folke/lazy.nvim | 21,630 | Plugin Manager | `lua/lazy/`, `doc/lazy.nvim.txt`, `lua/lazy/health.lua`, LuaRocks, own site |
| nvim-telescope/telescope.nvim | 19,814 | Fuzzy Finder | `lua/telescope/`, `plugin/telescope.lua`, `doc/telescope.txt`, health, LuaRocks |
| nvim-treesitter/nvim-treesitter | 14,439 | Syntax | `lua/nvim-treesitter/`, three `plugin/*.lua`, `doc/nvim-treesitter.txt`, health |
| neovim/nvim-lspconfig | 13,968 | LSP | `lua/lspconfig/`, `plugin/lspconfig.lua`, three doc files, health |
| mason-org/mason.nvim | 10,502 | LSP | `lua/mason/`, `doc/mason.txt`, health |

The common pattern is structural, not stylistic: one dedicated
Neovim-branded repository per plugin, Neovim named on the first
screen, installer snippets, vimdoc under `doc/`, a `:checkhealth`
module, a license, and a one-line list entry. None link a monorepo
subdirectory.

### The decision: a dedicated mirror repository

`clients/nvim` already has the exact runtime layout Neovim's own
`lua-plugin.txt` prescribes (`ftdetect/ ftplugin/ syntax/ lua/`), a
license, an active train, and a published LuaRocks rock. What it could
not satisfy was presentational: a landing page of its own. So the
channel gains `upyesp/epher.nvim`, a synced mirror repository on the
LSP-epher precedent (a registry mirror, not a separately versioned
product; ADR-0066's version-locking is preserved and the mirror's tags
mirror the monorepo's `v*` tags):

- `clients/nvim/listing/assemble-repo.sh` assembles the tree (runtime
  directories, `lua/`, `doc/epher.txt`, `images/`, the listing copy of
  the README cut at the `## Maintainer note` divider, LICENSE).
- `.github/workflows/epher-nvim-publish.yml` syncs the tree and the
  matching tag on every `v*` push, gated on `NVIM_PACKAGE_TOKEN` in
  the `stores` environment.
- `clients/nvim/lua/epher/health.lua` and `clients/nvim/doc/epher.txt`
  were added to the client itself, so the mirror, the zip, and the
  rock all carry them.

The fresh awesome-neovim PR (title `Add \`upyesp/epher.nvim\``, same
compliant entry line, Languages section) is deliberately held until
the mirror has been retested by hand; the repo's one-week age
requirement starts at creation, so opening the PR later costs nothing.

### Monorepo-root visibility (fixed for every channel)

The root README now carries an "IDE extensions" section: an
editor-to-install-source table naming Neovim (the mirror, plus the
rock), Vim (vim.org), Sublime (LSP-epher), and the rest. This is the
literal rejection reason, fixed at the surface every reviewer, every
directory crawler, and every store listing lands on.
