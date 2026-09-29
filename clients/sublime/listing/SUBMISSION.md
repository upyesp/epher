# Submitting LSP-epher to Package Control

The channel for LSP helper packages is `sublimelsp/repository`, the LSP
team's repository file, which is included in Package Control's default
channel. There is no packagecontrol.io account and no submission form:
publication is a pull request against that repository, and after it is
merged every semver tag on `upyesp/LSP-epher` becomes a release within
about an hour with no further PR
(`docs/research/sublime-packagecontrol-publishing.md`).

Artifacts in this directory:

- `package-control-entry.json`: the exact object to add to
  `repository.json` in `sublimelsp/repository`.
- `assemble-repo.sh`: builds the standalone package repository content
  from this monorepo into a target directory.

## Secrets

The PR itself needs no secret: the identity is the GitHub account.

Later CI has one optional secret, and only because the package must
live in its own repository: a fine-grained GitHub PAT with
`contents: read/write` on `upyesp/LSP-epher`, stored as
`SUBLIME_PACKAGE_TOKEN` in the `stores` environment. That PAT is for
the sync workflow sketched in the research doc, which does not exist in
this repo yet. Nothing in the first publication waits on it.

## Step 1: assemble and create upyesp/LSP-epher (human, one time)

1. Assemble the tree:

   ```sh
   clients/sublime/listing/assemble-repo.sh /tmp/LSP-epher
   ```

   The script regenerates `epher.tmLanguage` from the shared grammar
   and copies exactly the package files: `epher.py`,
   `LSP-epher.sublime-settings`, `LSP-epher.sublime-commands`, the
   three `Default (<platform>).sublime-keymap` files,
   `epher.tmLanguage`, `README.md`, `LICENSE`, and `.python-version`.
   The last one is load-bearing: without it the plugin loads on Sublime
   Text's legacy python 3.3 host and the import fails (see
   `listing/NOTES.md`).

2. The README that ships is already the listing copy of
   `clients/sublime/README.md`: everything above its "## Maintainer
   note" divider (the Package Control install path, the
   `epher-lsp` binary step, the inlay-hints setting, absolute image
   URLs, no social media links). The assembler cuts the divider; the
   maintainer notes below it stay in the monorepo. Nothing to adapt
   by hand.

3. Create the public repository `upyesp/LSP-epher` (human) and set its
   GitHub metadata deliberately, because the listing is built from it:
   - `description`: the one-line listing text, for example
     "A calculator language where every statement's answer appears
     inline, for Sublime Text via the LSP package";
   - homepage: `https://epher.org`;
   - issues enabled.

4. Copy the assembled tree into the repository root, commit, and push.

5. Tag a semver release **before** opening the channel PR: "a valid
   semver numbered tag must exist on the repository", and branch-based
   releases are not accepted for new packages.

   ```sh
   git tag v0.5.57
   git push origin main --tags
   ```

6. Cleaning: the assembler ships only package files, so there is
   nothing to `export-ignore` today (the research doc's `.gitattributes`
   step targets helper files, tests, and images, none of which are in
   the assembled tree). If anything else is ever added to the
   repository, add `.gitattributes` with `export-ignore` for it, since
   Package Control builds the install archive with `git-archive`.

## Step 2: the sublimelsp/repository PR (human, one time)

1. Fork `sublimelsp/repository`.
2. Add the object from `package-control-entry.json` to its single
   `repository.json`, in alphabetical position, without reformatting
   the existing entries. The object is:

   ```json
   {
   	"name": "LSP-epher",
   	"details": "https://github.com/upyesp/LSP-epher",
   	"labels": ["lsp", "epher", "language syntax"],
   	"releases": [
   		{
   			"sublime_text": ">=4132",
   			"tags": true
   		}
   	]
   }
   ```

   `"tags": true` is the whole update mechanism: every new tag is a
   release.

   The `sublime_text` floor is the one real decision in the entry. It
   is `>=4132` here because the client cannot work without the LSP
   package and this matches LSP's own floor in the same channel; the
   client needs the LSP release series that renders inlay hints and
   semantic tokens (the capture rig ran ST build 4215 with LSP
   `4070-2.14.0`). The research doc's example entry used `>=4000` and
   flags the floor as a decision to make from the LSP dependency, so if
   the LSP floor moves, this single value moves with it.

3. Open the PR with the title **"Add LSP-epher"** (the channel's
   conventions ask for an alphabetical insert and no reformatting).
   `sublimelsp/repository` runs automated reviewers on every PR
   (`st-schema-reviewer-action` and `st-package-reviewer-action`)
   before a human looks.

4. Conduct the review personally. The channel is explicit that a human
   assesses the author's willingness to maintain the package long
   term, and that bot-opened PRs are not the point. One package per
   PR; review takes weeks, in maintainers' spare time.

## After the merge (automatic, forever)

- Every new semver tag on `upyesp/LSP-epher` becomes a release. The
  crawler re-reads the channel sources about once an hour, so a pushed
  tag is live within the hour with no PR and no human step.
- A version is listed once a tag exists; same-version republishing does
  not exist. Never re-point a tag; always bump with the release train.
- The per-train flow is: sync the client files to `upyesp/LSP-epher`,
  push, tag `v<version>`, and the listing follows within the hour.

## If the CI sync is wired later

The research doc sketches `.github/workflows/sublime-publish.yml`: on
each train it regenerates the grammar, syncs the package files plus
`README.adapted.md` and `LICENSE` into `upyesp/LSP-epher`, pushes, and
tags `v<version>`. It is gated on `SUBLIME_PACKAGE_TOKEN` in the
`stores` environment and skips with a notice while the secret is
absent. Creating that PAT is a human one-time step (GitHub settings,
fine-grained, `contents: read/write` on `upyesp/LSP-epher` only); it is
not needed for the CHANNEL PR above.
