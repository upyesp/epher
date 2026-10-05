# Publishing epher on vim.org (Vim Scripts)

The channel is the official Vim Scripts index at vim.org/scripts. It
has no API and no tokens; the version upload is driven through the
site's own web forms by CI (`vim-publish.yml` + `publish-vimorg.py`,
since 2026-10-05 — see docs/research/vim-org-publishing.md for the
analysis). The GitHub release zip is the artifact side and stays
automated.

## Status

Published 2026-09-30 as script_id 6195:
https://www.vim.org/scripts/script.php?script_id=6195

The first version is the lean package described below, version 0.5.57,
Vim compatibility 9.0. The listing copy is `DESCRIPTION.md`: the text
above the separator fills the description field, the install block
fills the separate install-details field. The per-field form values
are recorded out-of-repo in `/home/pete/epher-vim-org-fields.txt`,
like the Eclipse fields file.

## The upload size rule (learned 2026-09-30)

vim.org silently rejects packages over its upload cap: the form
re-renders blank with no error, and nothing is saved. The full GitHub
`epher-vim.zip` (352K, carries images and listing) trips it; the
largest package the site hosts today is 163K. vim.org gets a lean zip
instead: `epher-vim/` with `ftdetect/`, `syntax/`, `ftplugin/`, and
`README.md` only, 8K. Build it from the release tag:

    cd /home/pete/code/epher
    stage=epher-vim; rm -rf "$stage"; mkdir -p "$stage"
    cp -r clients/vim/ftdetect clients/vim/syntax clients/vim/ftplugin \
       clients/vim/README.md "$stage/"
    zip -qr /home/pete/epher-vim.zip "$stage"; rm -rf "$stage"

## Per train (CI, since 2026-10-05)

`vim-publish.yml` fires on every `v*` tag, builds the lean zip from the
tag, and runs `clients/vim/listing/publish-vimorg.py`: log in, parse
the upload form, echo every field back with the release overrides
(version = tag without the leading `v`, Vim version 9.0, release notes
= this train's `clients/vim` commit subjects, with a plain fallback
line when the train touched nothing in the client), POST, then verify
the script page shows the new version. Re-running an already-published
tag is a no-op; without credentials the job posts a notice and skips.

One-time setup (the only human step left): add the vim.org account
credentials that own script_id 6195 to the repo's `stores` environment
as `VIMORG_USERNAME` and `VIMORG_PASSWORD`. Nothing else is secret —
the script id is public page data.

The pending manual 0.5.59 upload became unnecessary: CI publishes the
next tag's version directly (the versions table will simply not have a
0.5.59–0.5.63 row unless one final manual upload fills it).

Manual fallback if CI is red: "upload new version" on the script page
(`add_script_version.php?script_id=6195`), lean zip rebuilt per the
recipe above, and confirm the versions table shows the new row.

There is nothing else to maintain: no moderation queue, no listing
edits unless the description changes (still a browser form, same
account).
