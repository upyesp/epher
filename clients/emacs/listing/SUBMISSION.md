# Submitting epher.el to MELPA

The recipe text is in `recipe` next to this file. It is the exact
recipe from `docs/research/emacs-melpa-publishing.md`:

```elisp
(epher
 :fetcher github
 :repo "upyesp/epher"
 :files ("clients/emacs/epher.el"))
```

`:fetcher github` and `:repo "upyesp/epher"` are the dedicated forge
fetcher the MELPA README prefers; `:files` paths are relative to the
repository root, and the file lands as `epher.el` in the built package.

## Secrets

There are none. MELPA has no channel account to create, no API key,
no token, and no CI job. The GitHub account that opens the PR is the
only credential involved. Nothing belongs in the `stores` environment
for it, and no `.github/workflows` file exists or is needed: the whole
channel is a GitHub pull request and a review conversation.
Everything below is human.

## Before the PR (human, one time, in this repo)

The recipe can be submitted as soon as `clients/emacs/epher.el`
passes the MELPA requirements. The repo-side preparation is already
done in this tree:

1. `epher.el` carries an `Author:` header and the MIT license notice
   above `;;; Commentary:` (the MELPA package page exposes authors,
   and CONTRIBUTING.org asks for the boilerplate in the header). Add
   an `Assisted-by:` line under `Author:` if LLMs generated some of
   the code.
2. The Commentary is rewritten as the listing copy: plain text, no ADR
   numbers, adapted from `clients/vscode/README.md`. MELPA renders the
   Commentary as preformatted text (`epher-readme.txt`); markdown,
   links, and images never render, so the screenshots live in
   `clients/emacs/images/` and `clients/emacs/README.md` instead.
3. Keep `Version:` in lockstep with the train; it is `0.5.57` today.
4. The package must have lived in a public repo for one month or
   more; `upyesp/epher` already clears this.

Remaining human checks before the PR:

- run `package-lint`, `checkdoc`, and a byte-compile on `epher.el`
  and fix what they flag (the research doc's one open item);
- confirm the repository has been public for at least a month (yes)
  and that no `epher-pkg.el`, README, or changelog ships inside the
  package (`:files` ships exactly one file, so this holds).

## The one-time PR

1. Fork `melpa/melpa` and create a branch, for example
   `add-recipe-epher`.
2. Add the recipe at exactly `recipes/epher`. The file name must equal
   the package name (`epher`), not `recipe`; `recipe` here is only the
   staged copy. No `epher-pkg.el` is created by hand; MELPA generates
   it from the package headers.
3. Test locally, per CONTRIBUTING.org:
   - `make recipes/epher` builds the package into `packages/`;
   - `MELPA_CHANNEL=stable make recipes/epher` checks that the
     `v0.5.57` tag parses through the default `:version-regexp`;
   - `make sandbox INSTALL=epher` gives a sandboxed Emacs with the new
     package installable.
4. Commit and open the PR with the title **"Add recipe for epher"**
   (the template's own instruction). Fill the template: brief summary,
   a direct link to `https://github.com/upyesp/epher`, your
   association (maintainer), "Relevant communications with the
   upstream package maintainer" as *None needed*, and the checklist
   (GPL-compatible license, package-lint clean, byte-compiles,
   checkdoc clean, built and installed, repo public for at least a
   month).
5. One reviewer question is expected: CONTRIBUTING.org asks for a
   dedicated SCM repository per package. epher is a version-locked
   monorepo by ADR-0066; the `:files` spec works, and the shared
   `v0.5.x` tags keep every client on the same version by
   construction. Answer with the ADR; the guideline is explicitly
   evaluated case by case.
6. MELPA CI runs on the PR itself and builds the changed recipe; fix
   anything it reports. A maintainer then reviews, typically a week or
   several.

## After the merge (automatic, forever)

- MELPA's build server checks out the recipe's repo, builds
  `epher-<date>.el`, generates `epher-readme.txt` from the Commentary,
  and updates `archive.json` and the download counter, at intervals
  through the day.
- Unstable MELPA, the default archive, versions by the date of the
  last commit that touched `clients/emacs/epher.el`, so every commit
  on main becomes a new package build within a day.
- MELPA Stable versions from the repo's tags, and the per-train
  `v0.5.x` tags already parse, so stable follows `0.5.57`, `0.5.58`,
  and so on with no action from us.
- There is no per-release action on MELPA, ever. The recipe changes
  only if the file path changes.

## NonGNU ELPA (the optional second queue)

NonGNU ELPA is worth one email after MELPA settles: it is the only
channel that puts epher into `M-x list-packages` with no user
configuration (the default archive since Emacs 28.1), and unlike GNU
ELPA it requires no FSF copyright assignment. It is submitted by
email, not by PR:

1. Subscribe to `emacs-devel@gnu.org` first (an unsubscribed address
   needs manual approval).
2. Send a short message to the list: what the package is, a link to
   the public git repository, and whether GNU ELPA or NonGNU ELPA is
   preferred (state NonGNU).
3. Mention that the Lisp file is not at the repository root (it is at
   `clients/emacs/epher.el`); the submission guidance asks for exactly
   that.
4. The description comes from `README.org` if one exists, but Markdown
   is excluded by policy, so the `;;; Commentary:` serves here too.
   The same rewritten Commentary from the MELPA work is the listing
   copy.
5. Volunteers review on the mailing list; the queue is measured in
   emails, not CI minutes. Do not block the MELPA PR on this.

Neither archive asks for a credential of any kind.
