# VS Code Marketplace page anatomy: what the popular extensions do

Research of 2026-09-16, in preparation for the epher extension's
marketplace readiness. Question: what do the most-installed extensions
put on their marketplace page, and what should epher adopt?

## Method

Fetched the marketplace items pages for three permanently
top-installed extensions of different kinds (a language platform, a
formatter, a git superpower) and catalogued their readme structure and
image use:

- **Python** (`ms-python.python`), readme headings: intro → support
  for vscode.dev → installed extensions → extensibility → **quick
  start** → useful commands → feature details → locales → questions →
  data and telemetry. 12 images, 4 of them animated GIFs.
- **Prettier** (`esbenp.prettier-vscode`), one-line intro →
  **installation first** → configuration → usage → linter integration
  → workspace trust → settings → troubleshooting. Images immediately
  under the intro.
- **GitLens** (`eamodio.gitlens`), hero value-prop line ("Git
  supercharged") → getting started → one section per feature, **each
  with its own screenshot or GIF**. 12 images.

## The common anatomy

1. **Icon** (128×128 minimum PNG) and a `galleryBanner` color that
   match the brand; the page header is tinted with it.
2. **The first screen sells**: hero visual (screenshot or GIF) within
   the first viewport, above any installation text. GitLens and Python
   both lead with imagery; Prettier's single image sits at the top.
3. **Install instructions are early and short**: one numbered list,
   no preconditions before it. Prettier makes it the first heading.
4. **One feature, one visual**: moving features get GIFs (typing,
   completions, answers appearing); static features get PNGs.
5. **A settings/requirements section**: even when the answer is
   "nothing to configure" (Prettier's settings table; Python's data
   and telemetry statement).
6. **Support/contribution footer** with links; license stated.
7. **Metadata hygiene**: ≤200-char description, 3–7 search keywords,
   1–3 categories, repository field, badges (version + CI are the
   common pair).

## What epher adopted (2026-09-16)

- Icon: the existing epher mark (dark tile, white `e`) at 256 px;
  galleryBanner `#1c1c1e` dark, the page header matches the icon.
- Hero: a real screenshot of the extension working (inlay answers
  next to `6371 km` / `disc(2, 7, 3)` / `55 mile/hr` in `km/hr`),
  captured from an actual VS Code 1.138.0 session driving the packaged
  extension, not a mockup.
- GIF: the script typed line by line with answers appearing
  (`images/demo.gif`, 9 frames, ~260 KB, captured the same way).
- Hover and completion each get their own screenshot.
- Install as an early numbered list (Quick start), the no-download
  story stated in one sentence, requirements ("no compiler, no
  runtime, no network"), an explicit no-telemetry paragraph (the
  Python-style data statement, in our case trivial), and
  version + CI badges.
- Developer/build content moved into a collapsed `<details>` block so
  the marketplace page stays user-facing.
- Categories: `Programming Languages`, `Snippets`. Keywords:
  calculator, calculator language, units, unit conversion, astronomy,
  physics, lsp.

## Authenticity note

All screenshots and the GIF are real output of the real extension:
captured from VS Code 1.138.0 (Electron, under Xvfb) with the packaged
vsix installed and the bundled wasm server answering, driven over the
Chrome DevTools Protocol; only dead UI space (an empty side panel) is
cropped. The demo script evaluates exactly as shown (verified with the
CLI before recording).
