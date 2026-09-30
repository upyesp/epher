# Publishing epher on vim.org (Vim Scripts)

The channel is the official Vim Scripts index at vim.org/scripts. It
has no API, no tokens, and no CI surface: every step is a human at a
browser (docs/research/vim-org-publishing.md). The GitHub release zip
is the artifact side and stays automated.

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

## Per train (human, one minute)

1. Cut the train as usual; GitHub gets the full `epher-vim.zip`
   automatically.
2. On the script page, "upload new version"
   (`add_script_version.php?script_id=6195`): upload the lean zip
   rebuilt from the release tag, version string = the tag without the
   leading `v`, release notes = this train's `clients/vim` changelog
   lines, Vim version 9.0.
3. Confirm the page's versions table shows the new row.

There is nothing else to maintain: no moderation queue, no listing
edits unless the description changes (also a browser form, same
account).
