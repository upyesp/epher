# Submitting epher to LuaRocks and awesome-neovim

The Neovim client's distribution channel is luarocks.org, the registry
rocks.nvim installs from with `:Rocks install epher`, plus the
awesome-neovim discovery list (neovimcraft scrapes that list
automatically). There is no first-party Neovim store
(`docs/research/neovim-publishing.md`).

What this repo carries for the channel:

- `clients/nvim/` is now the standard Neovim plugin layout: the Lua
  module in `lua/`, plus `ftdetect/`, `ftplugin/`, and `syntax/` synced
  from `clients/vim/` by `clients/nvim/sync-runtime.py`. The rock is
  cut from these copies; `clients/vim/` stays untouched, so every
  existing runtimepath setup keeps working.
- `clients/nvim/epher-scm-1.rockspec` is the release template the
  action consumes. It is not a local development rockspec: the
  placeholders are substituted by the action.
- `.github/workflows/luarocks-publish.yml` is the tag-triggered
  upload. It is deliberately not wired into `release.yml`: it runs on
  the `v*` tag itself.

## Layout decision, stated plainly

The research doc recommends syncing the runtime files into
`clients/nvim` so the rock action's defaults work. One detail needs a
release template rather than pure defaults: the action fetches the
tag's GitHub archive and builds from the archive root, and its
generated rockspec points `source.dir` there, where no `lua/` or
runtime directories exist in the monorepo. The template in
`clients/nvim` is the action's generated template with exactly one
change: `source.dir` points at `clients/nvim` inside the archive. The
plugin layout itself is the doc's option 1: `lua/` by the builtin
convention, the runtime directories via the action's default
`{{ neovim.plugin.dirs }}` copy list (only directories that exist are
copied). The workflow passes explicit labels, summary, and detailed
description, as the doc prescribes.

The template lives next to the client it packages
(`clients/nvim/epher-scm-1.rockspec`), not at the monorepo root: it is
only meaningful for the `clients/nvim` build, and the workflow hands
its path to the action's `template` input.

The workflow also runs `sync-runtime.py --check` on every tag, so a
stale copy of a `clients/vim` edit cannot ship inside a rock.

## Human-only steps (one time)

1. **Create a luarocks.org account** at
   `https://luarocks.org/`. GitHub login works; the account name is
   the uploader shown on the rock page
   (`https://luarocks.org/modules/<account>/epher`).
   Done 2026-09-30: the account is `upyesp`, verified live
   (`https://luarocks.org/modules/upyesp` returns 200).

2. **Generate a fresh API key** at
   `https://luarocks.org/settings/api-keys` (Settings, API keys,
   Generate New Key). The September 2026 security incident revoked
   every API key on the server and ended every session, so the key
   must be minted after 2026-09-26; any older key is dead. There is no
   other way to get a working key.
   Done 2026-09-30: a fresh key was minted and stored.

3. **Store the key as `LUAROCKS_API_KEY`** in the `stores`
   environment secret. Nothing else in the channel uses a token.
   Done 2026-09-30: `gh secret set LUAROCKS_API_KEY --env stores`
   (the environment lists it alongside OVSX_PAT and the others).
   The first upload fires on the next `v*` tag, which is the 0.5.59
   train; the rock series therefore starts at 0.5.59-1, since the
   workflow did not exist at the `v0.5.57` tag and tags are never
   re-pointed.

   Timing matters, because the tag is the trigger: the workflow was
   designed to run on every `v*` tag, and while the secret is absent
   it prints a notice and skips. Set the secret before the next
   release train tag. If a tag is already pushed without the secret,
   that train simply has no rock; the next train uploads its own
   version. Never re-point a tag to backfill, and never force a
   same-version upload: luarocks rejects a version that already
   exists, and the action no-ops duplicates by default.

## The awesome-neovim PR (human, one time)

Opened 2026-09-30 as https://github.com/rockerBOO/awesome-neovim/pull/2532
from the fork branch `add-epher`, titled "Add `upyesp/epher`" with the
template checklist ticked; the entry sits at the end of the main
Programming Languages Support list and all five repository CI checks
(terminology, typos, title, formatting, awesome linter) are green.
Awaiting maintainer merge; static thereafter.

1. Fork `rockerBOO/awesome-neovim` and add one line to the
   **Programming Languages Support** section:

   ```
   - [upyesp/epher](https://github.com/upyesp/epher) - Calculator language with answers inline, LSP server glue, and syntax for `.epher` scripts.
   ```

   The description ends with a period, uses no emojis, and contains
   neither "plugin" nor "Neovim" (the list's own rules).

2. PR rules: the title must match
   `` Add|Update|Remove `username/repo` `` with the backticks, so
   ``Add `upyesp/epher` ``; one plugin per PR; use the PR template
   checklist (licensed, line ends with a period, acronyms
   capitalized). Run `./scripts/readme-check.sh` locally before
   pushing.

3. Acceptance: the entry must be Neovim-specific, functional,
   open-source licensed, have a quality README with install and usage,
   and be at least a week old. epher clears all of them already.

4. After the merge there is nothing to maintain: the entry is static
   text and never changes on a release train; neovimcraft picks it up
   by scraping.

5. Optional, from the research doc: set the repository's GitHub social
   preview to the epher monogram, since the rock page and the
   awesome-neovim PR are previewed from the repo. The monogram has no
   surface inside the rock itself (the rockspec format has no icon
   field).

## What CI does per release train

- A `v0.5.57` tag runs the workflow: it checks the synced runtime
  copies, then `lumen-oss/luarocks-tag-release@v7` generates the
  rockspec from `clients/nvim/epher-scm-1.rockspec`, runs a local
  install test, uploads rock `0.5.57-1` (the leading `v` is stripped;
  `modrev` 0.5.57, `specrev` 1), then tests an install from the server.
- The upload carries the `neovim` label, which lists the rock on
  `https://luarocks.org/labels/neovim`, and the summary and detailed
  text above.
- The rock page renders only that text: no README, no markdown, no
  images, no icon. The visuals live in `clients/nvim/README.md`.
- Re-running the workflow for an already-published version does
  nothing and stays green (`fail_on_duplicate` is left at its default,
  false).
- Nothing else in the channel is automated because nothing else needs
  to be: awesome-neovim is static after the merge, and neovimcraft is
  an automatic scrape.

## Verify by hand after the first upload

```sh
luarocks search epher            # the published versions
:Rocks install epher             # in Neovim with rocks.nvim
:checkhealth rocks
```

The rock installs the module and the runtime files; `setup({ cmd = ... })`
remains the user's call, as `clients/nvim/README.md` documents.
