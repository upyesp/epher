# ADR-0024: Save dialogs, the 0x keypad bank, and TUI menu paint order

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Four reports arrived together:

1. **The keypad looked incomplete against the guide.** The exactness and
   base conversions (`frac`, `dec`, `big`, `bin`, `oct`, `hex`) were
   buried at the end of the number-theory bank, and the factorial
   postfix `!` — the one operator without a key — had none. Users
   comparing the keypad to the guide's function tables concluded buttons
   were missing.
2. **The status line under the entry was slightly too large.**
3. **Save history / Save script used browser downloads** — the file
   landed in the default Downloads folder under a fixed name, with no
   way to choose a location or name. Not how desktop apps behave.
4. **The TUI's menus rendered under screen content.** When history or
   graph text already occupied the area a menu drops into, items were
   hidden — ratatui paints widgets into one shared buffer in call
   order, and the menu was drawn before the panels that overlapped it.

## Decision

- **A dedicated 0x bank (web and TUI).** `frac`, `dec`, `big`, `bin`,
  `oct`, `hex`, and the `!` postfix move into their own bank, labelled
  `0x` — the language's own base notation, unlocalized like every other
  language token (ADR-0007). The web keypad's tab carries a localized
  aria-label (`keypad-tab-conv`, all eight locales); the TUI's bank row
  shows the literal `0x` like its sibling banks. The number-theory bank
  keeps gcd/lcm/mod/factorial and the statistics. Guide §5.2 lists the
  new bank in every locale. The TUI's number-theory bank shrinks from
  five rows to three, which also fixes a latent bug: five rows did not
  fit the seven-row keypad area, so its last row (bin/oct/hex) was
  clipped.

- **Save history / Save script show the operating system's save
  dialog.** Desktop (Tauri): a new `save_file_dialog` IPC command uses
  tauri-plugin-dialog's native save dialog (parented to the main
  window, `epher`/`.epher` filter, suggested file name) and writes the
  file at the chosen path; the status line reports the path. The dialog
  call and the write stay in one command so there is no window between
  picking and writing; the write itself is a separate function covered
  by a unit test. PWA: the browser's own save picker (File System
  Access API `showSaveFilePicker`, Chromium) with the same suggested
  name and type; where the API is absent (Firefox, Safari) the app
  falls back to the previous download behavior and says so. Cancel is
  silent in every path — native apps do not announce a cancelled
  dialog. Save script with an empty entry now says "nothing to save"
  instead of closing silently. Capability: `dialog:allow-save`.

- **The status line shrinks 20%** (1.6rem → 1.28rem).

- **The TUI draws its menu popup last, over every panel.** A `Clear`
  widget first blanks the popup's rectangle, then the menu renders into
  it — after history, keypad, graph, and hints, so nothing can paint
  over the menu again. A regression test fills the history and asserts
  the open File menu contains its items and no history text; it fails
  on the old draw order and passes on the new one.

## Alternatives considered

- **`tui-menu` crate for the TUI menus.** Rejected. The crate has been
  unmaintained since 2022, its items take `&'static str` labels (our
  menus are localized at runtime across eight locales), and its
  navigation model does not match the APG menubar behavior (arrow-key
  wrapping, menu switching, Escape, item activation) that epher's menus
  already implement and test. The reported defect was a draw-order bug,
  not a missing menu abstraction — moving the render fixed it.
- **PWA-only picker without a fallback.** Rejected; Firefox and Safari
  users would lose the feature entirely. The download remains the
  fallback and keeps the honest "saved to your downloads" message.
- **Bigger keypad banks instead of a new tab.** Rejected; the
  conversions are their own conceptual group, and the TUI bank was
  already overflowing its fixed area.

## Consequences

- Desktop save flows need the dialog plugin's capability; the command is
  thin and the write path is unit-tested, while the webview branch is
  covered by browser tests stubbing `__TAURI__` and the picker.
- The guide's keypad sentence names five banks; tests asserting the
  old bank order were updated.
- TUI menus now behave as expected on any terminal content; the popup
  render sits at the end of `draw` with a comment explaining why it
  must stay there.
