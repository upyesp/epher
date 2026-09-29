# Submitting epher to the Zed extension registry

The official channel is the Zed Extension Registry, browsed at
`https://zed.dev/extensions/epher` and in the editor's Extension
Gallery. There is no developer portal, no Zed account, and no
publishing token: first publication is a pull request to
`zed-industries/extensions`, every later version is another PR, and
Zed's own CI does all packaging and publishing after merge
(`docs/research/zed-publishing.md`).

Artifacts in this directory:

- `extensions-entry.toml`: the PR payload. It carries the submodule
  setup, the `extensions.toml` fragment, and the `[grammars.epher]` pin
  with its blocking dependency.
- `../LICENSE`: added for this channel. Zed's license rules require an
  accepted license file **inside the extension's path**, not merely at
  the repository root, and as of 2025-10-01 CI fails without it.

## Secrets

- First publication needs none beyond the GitHub account. The review
  conversation is human and the 3-week responsiveness rule is real.
- Later update PRs use one secret: `ZED_EXTENSIONS_TOKEN`, a GitHub
  personal access token with `repo` and `workflow` scopes that can push
  to the `upyesp` fork of `zed-industries/extensions` and open the PR
  upstream. Store it in the `stores` environment. The research doc
  sketches the update workflow, which does not exist in this repo yet;
  nothing in the first PR waits on it.
- Zed's CI uses its own credentials after merge; no contributor
  credential is involved at any publishing step.

## Before the PR (repo work)

1. **License**: done. `clients/zed/LICENSE` is a copy of the root MIT
   LICENSE, which is one of the accepted licenses. The rule is about
   the file living in the extension directory, and a copied file
   satisfies it the same way a symlink does.

2. **Grammar pin**: in progress, blocked. Zed requires a
   `[grammars.epher]` entry for every language a language extension
   provides, and the entry must pin a real repository: Zed's packaging
   pipeline clones the declared grammar repository and compiles it with
   wasi-sdk on Zed's CI. That repository,
   `https://github.com/upyesp/tree-sitter-epher`, does not exist yet.

   **Do not open the first-publication PR until it does.** A pin to a
   missing repository fails the merge-to-main packaging for the whole
   registry. The exact block to add to `clients/zed/extension.toml` is
   in `extensions-entry.toml`, Part 3. The current `extension.toml`
   deliberately has no grammar because a declared grammar breaks
   sandboxed dev-extension installs; that reasoning is dev-only and
   flips for the registry, where installs download a finished package.

3. **Version lockstep**: `clients/zed/extension.toml` and
   `clients/zed/Cargo.toml` both carry `0.5.57`; keep them moving
   together with `crates/cli`. The registry version is strict semver
   with no `v` prefix and must never decrease.

4. **Test at the exact commit you submit**: install the extension as a
   dev extension at the pinned submodule commit and verify the language
   server starts and the settings in the README work. Compile the wasm
   once (`wasm32-wasip2`) as a smoke test; `zed-extension` from
   Zed's `crates/extension_cli` is the packaging CLI Zed's CI pins.

5. **README split**: the registry listing renders no README, so the
   README's job is the repository audience. Move install-by-search
   first, keep the two settings to flip (semantic tokens, inlay hints)
   prominent, and keep social media links out. The listing text itself
   is the `description` string in `extension.toml`; no screenshots,
   icons, or categories exist to prepare.

6. **Prerequisites self-check**: id `epher` is unique, kebab-cased, and
   contains none of the reserved words; the description is English and
   names what it does; the extension does not duplicate an existing
   registry entry.

## The first-publication PR

1. Fork `zed-industries/extensions`, to a personal account if
   possible: Zed staff sometimes push fixes to a contributor's PR.
2. Add the submodule at exactly `extensions/epher`:

   ```sh
   git submodule add https://github.com/upyesp/epher.git extensions/epher
   ```

3. Add the `[epher]` table from `extensions-entry.toml` to
   `extensions.toml`, then run `pnpm sort-extensions` (CI enforces the
   sort of `extensions.toml` and `.gitmodules`).
4. Commit and open the PR. The PR template is a single checklist line:
   read the contribution guidelines. Rules: exactly one extension per
   PR, at most three open PRs, and a response to maintainer feedback
   within 3 weeks or the PR is closed (a fresh PR may be opened any
   time).
5. A human maintainer reviews; first feedback usually takes a few
   weeks, occasionally one to two months. Updates are held to the same
   standards as new submissions.

## After the merge (Zed's CI, not ours)

- On every push to main, Zed's `package-extensions` job compiles each
  extension whose version is not yet in its blob store with the pinned
  `zed-extension` CLI and uploads it. No contributor credential is
  involved.
- A version publishes exactly once. A same-version re-merge is a no-op,
  and there is no replace or delete mechanism for authors; a broken
  package is fixed with a new version and a new PR.
- The extension id is immutable, and the version must never decrease.

## Every later release train

One `v0.5.x` tag across all clients, then the registry update:

- The documented community automation is
  `huacnlee/zed-extension-action@v1` with `push-to: upyesp/extensions`
  and `COMMITTER_TOKEN` set to `ZED_EXTENSIONS_TOKEN`. On a tag push it
  updates the fork's submodule pointer and `extensions.toml` and opens
  the PR upstream.
- The official `zed-industries/zed-extension-action` no longer exists
  (its URL 404s); any guide naming it is stale.
- Manual fallback, per the upstream updating doc:

  ```sh
  git submodule update --remote extensions/epher
  # bump `version` in extensions.toml to the new semver
  pnpm sort-extensions
  ```

- The update workflow sketched in `docs/research/zed-publishing.md`
  also verifies lockstep and that the `v<version>` release carries the
  `epher-lsp` assets the extension downloads at runtime. It is gated on
  `ZED_EXTENSIONS_TOKEN` and skips with a notice until the secret
  exists.
