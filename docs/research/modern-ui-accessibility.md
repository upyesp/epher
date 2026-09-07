# Modern UI conventions and accessibility for the epher marketing site

**Goal:** establish research-backed guidance for the planned redesign of the
epher marketing site (`site/index.html`, `site/styles.css`): a new accent
color (not amber, not default blue, not purple), responsive desktop+mobile
layout with a hamburger menu, and About and Privacy pages — under the project's
hard constraint of WCAG 2.2 AA (`docs/accessibility.md`). Every recommendation
below traces to a primary source (W3C/WAI specs and Understanding docs, the
ARIA Authoring Practices, MDN, web.dev, IBM Carbon's design system) or to an
explicit computation; aesthetic judgments that no normative source governs are
marked as analysis.

**Method (all claims verified, not assumed):** consulted only first-party
documentation and read every cited page in full before quoting or paraphrasing
it. WCAG 2.2 requirements are quoted from the Recommendation itself and its
WAI Understanding documents; navigation-widget guidance from the WAI-ARIA
Authoring Practices Guide (APG) disclosure pattern and its navigation examples;
UI conventions from web.dev's Learn Responsive Design course and articles, MDN
reference pages, and IBM Carbon's design system (read both as published pages
and as the source markdown in Carbon's own GitHub repo); landing-page
structure from a field survey of eight first-party developer-tool marketing
pages fetched live and parsed for structure signals (h1, CTA, nav, code-in-
hero, footer, menu buttons). All contrast ratios were computed with the WCAG
relative-luminance formula (§5 shows the arithmetic); background luminance for
`#141416` is **L = 0.007070** (the design brief's "~0.005" is an
approximation). All pages accessed **2026-08-20**.

---

## 1. Modern UI conventions for technical/developer-tool marketing sites

### 1.1 Typography

**Fluid type via clamp(), bounded by rem.** web.dev's typography course is
explicit: viewport-relative `font-size` alone ("`html { font-size: 2.5vw; }`")
is a don't — "If you do, the user won't be able to resize the text" — and the
recommended pattern mixes a relative unit into the viewport term and clamps
it: `html { font-size: clamp(1rem, 0.75rem + 1.5vw, 2rem); }`, where "the
text size will never be smaller than 1rem or larger than 2rem" [22]. MDN
defines `clamp()` as clamping "a value within a range of values between a
defined minimum bound and a maximum bound" (min, preferred, max), and shows
the same heading idiom `clamp(1.8rem, 2.5vw, 2.8rem)` [19]. epher's current
hero already uses this shape (`clamp(2.5rem, 9vw, 4.25rem)`); the fixable
violation of the web.dev guidance is that its *middle* term is bare `9vw`
with no rem component — it should be e.g. `clamp(2.5rem, 1.5rem + 5vw,
4.25rem)` so user font-size scaling still participates (analysis applying
[22] to the existing CSS).

**Line length (measure): 45–75 characters, ~66ch implemented in CSS.** web.dev
quotes Bringhurst's *Elements of Typographic Style*: "Anything from 45 to 75
characters is widely regarded as a satisfactory line length for a
single-column page … The 66-character line (counting both letters and spaces)
is widely regarded as ideal", and translates it to CSS: there is no
line-length property, so cap the container — `article { max-inline-size:
66ch; }` — and "Don't set your line-lengths with a fixed unit like px … Use a
relative unit like rem or ch" [22]. WCAG 1.4.8 Visual Presentation (AAA)
gives the normative ceiling: "Width is no more than 80 characters or glyphs
(40 if CJK)" [13] — relevant because epher ships a zh-CN locale.

**Line height: unitless, ~1.5–1.65 for 66ch.** MDN: "The recommended line
height is around 1.5 – 2 … Use unitless values" [18-ref below; see MDN
Fundamentals [23]]. web.dev pairs the measure with the leading: 1.65 at 66ch,
2.0 at 45ch, because "if you use large line-height values for long lines of
text, it's hard for the reader's eye to move from the end of one line to the
start of the next line" [22].

**System font stack vs webfonts.** web.dev's position: web fonts "could
potentially degrade the user experience as it increases page load time. …
How fast those pixels get painted is a critical part of the user experience",
and if styling with the `system-ui` family "you might get all the benefits of
variable fonts without downloading any font files" [22]. MDN adds a caveat
epher must respect with its eight locales: `system-ui` "is intended to make
UI elements look like native apps, and not for typesetting large paragraphs of
text … For large paragraphs, use sans-serif or some other non-UI font family
instead" [20]. epher's current stack (system stack + `Noto Sans`, `Noto Sans
Arabic`, `Noto Sans Devanagari` fallbacks) is therefore already the
source-aligned choice for a multilingual, no-wasm marketing page: keep it
(analysis; consistent with [20], [22]).

**Type scale.** Carbon's guidance splits "productive" styles ("primarily used
within product spaces") from "expressive" styles (marketing surfaces) and
publishes a fixed scale from 0.75rem/12px to 5.75rem/92px, "built on a single
equation" [31]. A landing page needs only a 5–6 step subset (e.g. 0.875, 1,
1.25, 1.5, 2.25, clamp-scaled display) — analysis consistent with [31].

### 1.2 Layout

**Grids that adapt without breakpoints.** web.dev's card-grid idiom is
`.cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(15em, 1fr)); }`
— "the cards themselves automatically take up the right amount of space"
instead of hand-maintained breakpoints [25]; the same pattern extends to a
carousel-on-small/grid-on-large hybrid using flexbox + `overflow-x: auto` +
`scroll-snap-type: inline mandatory` under 50em, grid above [23-ui-patterns].
epher's `repeat(auto-fit, minmax(250px, 1fr))` cards already follow this
(250px ≈ 15em at default size). Content structure comes first: "make sure
that the flow of your content makes sense. This single column default
ordering is what smaller screens will get", and "Design for smaller screens
first" [23].

**Spacing on an 8px grid.** Carbon's 2x Grid: "The basic unit of 2x Grid
geometry is the 8-pixel square mini unit … Margin and padding are always
applied in fixed mini unit multiples" [35]. The companion spacing scale is
tokenized at 2, 4, 8, 12, 16, 24, 32, 40, 48, 64, 80, 96, 160px — "using
multiples of two, four, and eight" [32]. epher's current CSS mixes 1rem
steps and a few odd values (0.625rem/0.875rem paddings); aligning section
rhythm to the 2/4/8 ladder is the source-backed convention (analysis
applying [32], [35]).

**Measure vs container width.** Sources constrain the *reading* width (66ch,
§1.1) and the card grid (`minmax(15em, 1fr)`, §1.2) but no cited source
prescribes a px container maximum for a marketing page; epher's current
960px container is a defensible product decision, and widening it (e.g.
~1152px = 12 × 96) only for grid sections is analysis, not a source claim.

### 1.3 Color strategy

**Dark-mode-first with custom-property tokens.** web.dev's theming guidance:
store colors as custom properties and swap them in a `prefers-color-scheme`
(or `[data-theme]`) block "so you won't have to write all your selectors
twice"; declare `color-scheme` (e.g. `:root { color-scheme: light dark; }`)
so "the browser can provide the appropriate default styling for forms"; and
set `<meta name="theme-color">` per scheme with the `media` attribute [27],
[29]. epher already does all three; keep them.

**Avoid pure white text/surfaces on dark.** web.dev's dark-mode best
practices: "to prevent glowing and bleeding against the surrounding dark
content, I choose a slightly darker white. Something like rgb(250, 250, 250)
works well"; photographic/hero imagery should be dimmed or desaturated in
dark mode [29]. WCAG adds no rule here — this is comfort, not conformance
(ratios in §5 use exact values regardless).

**OKLCH for palette derivation.** MDN: `oklch()` "expresses a given color in
the Oklab color space … the cylindrical form of oklab(), using the same L
axis, but with polar Chroma (C) and Hue (h) coordinates" [21]. The practical
consequence for a design system: OKLCH's perceptually-uniform lightness means
steps of L hold apparent contrast constant across hues, so hover/pressed
variants can be derived as fixed L/C deltas. Browser support is universal
among evergreen engines (MDN compatibility table [21]).

**Accent usage discipline.** Carbon's rule of one: "The core blue family
serves as the primary action color across all IBM products and experiences.
Additional colors are used sparingly and purposefully" [34]. For epher: one
accent for CTAs, links, focus rings, and key graph strokes; everything else
neutral — matching the existing `--accent` discipline (analysis applying
[34]).

### 1.4 Surface treatment

**Boundaries are an accessibility feature, not just styling.** WCAG 1.4.11
requires "visual information required to identify user interface components
and states" to reach 3:1 against adjacent colors [1], [4]. The Understanding
doc's canonical passing example is "a standard text input with a grey border
(#767676) and white adjacent color" [4] — epher's `--border: #76767a` is
within 1/255 per channel of that example.

**Layered surfaces.** Carbon distinguishes surfaces with layer tokens
(`layer-01`, `layer-accent-01`, …) rather than relying on shadow alone
[34]; epher's panel + border + `--shadow` stack is the same model. If text
ever sits on a gradient or image: "make sure the text color meets contrast
standards in all places it appears" [36] — i.e., validate contrast at the
worst point of the gradient, not the average.

**When gradients read as dated.** No cited normative source rules on gradient
fashion. Field evidence (2026-08-20 fetches): Stripe still ships its angled
gradient hero [42]; Linear and Vercel use restrained ambient glows [43],
[44]; none of the developer-first pages (Node, Deno, Bun, Tailwind, Rust)
uses a gradient hero at all [37–41]. web.dev endorses gradients only for a
functional cue — the "gradient over the edge where content is truncated" in
the nav overflow pattern [23]. Analysis: full-bleed purple-blue mesh
gradients behind hero text carry the 2022–24 AI-landing-page cliché and age
fastest; localized glows or flat tokens do not. This is judgment, flagged as
such.

### 1.5 Motion

**Durations and purpose.** Carbon tokens the whole range of UI motion:
`duration-fast-01` 70ms ("micro-interactions such as button and toggle"),
`fast-02` 110ms (fade), `moderate-01` 150ms, `moderate-02` 240ms
(expansion/toast), `slow-01` 400ms, `slow-02` 700ms (background dimming),
with "productive" motion for function and "expressive" reserved — "Reserve
expressive motion for occasional, important moments" — and an evaluation
checklist that starts "Is your motion purposeful? What problem is motion
solving?" [33].

**Micro-interactions over scroll choreography.** web.dev's interaction
guidance: adapt target size to `pointer: coarse`, combine `:hover` with
`:focus` ("it's a good idea to combine :hover and :focus styles to cover
both interactions"), and "Don't use hover to hide important information" [24].
Scroll-reveal (content animating in on IntersectionObserver) is common in
the field but every cited source that mentions it treats it as risk to be
gated: see §6.

---

## 2. Landing page structure for technical products (field survey)

Survey of eight first-party marketing pages, fetched 2026-08-20 and parsed
for structure signals (h1 text, primary CTA link, `<code>`/`<pre>` counts in
the served HTML, footer link count, menu-toggle buttons):

| Site | H1 (served) | Primary CTA | Code in hero | Footer links |
|---|---|---|---|---|
| nodejs.org [37] | "Run JavaScript Everywhere" | "Get Node.js®" → /en/download | 5 `<code>`, 5 `<pre>` | 24 |
| deno.com [38] | "Better, faster JavaScript" | Docs + Deploy CTAs | 41 `<code>`, 8 `<pre>` | 27 |
| bun.sh [39] | "Bun is a fast JavaScript runtime & toolkit. All in one." | "Install Bun v1.4.0" → /get | 70 `<code>`, 12 `<pre>` | 22 |
| tailwindcss.com [40] | "Rapidly build modern websites without ever leaving your HTML." | "Get started" → docs/installation | 13 `<code>`, 7 `<pre>` | 32 |
| rust-lang.org [41] | "Rust" | "Get Started" → /learn | 0 | 14 |
| vercel.com [43] | "Agentic Infrastructure" | "Start now" / "Contact sales" | 0 | 80 |
| linear.app [44] | "The product development system for teams and agents" | "Sign up" / "Open app" | 4 `<code>` | 43 |
| stripe.com [42] | "Financial infrastructure to grow your revenue…" | "Start now" / "Contact sales" | 0 | 84 |

Convergent structure (analysis of the table; each row is a fetched source):

- **Hero = one-line value prop + one primary CTA.** Every page compresses
  the offer into a single sentence and pairs exactly one action with it
  (download for tools, signup for SaaS). Two CTAs max (secondary is text-only
  or "Contact sales") [37–44].
- **Developer-first pages put code in the hero; sales-led pages do not.**
  Deno/Bun/Tailwind ship dozens of `<code>` elements on the landing page;
  Vercel/Stripe ship zero. epher is developer-first: a real, runnable input→
  output example (the calculator's own idiom) is the field convention [38–40].
- **The primary CTA is concrete, not "Learn more."** "Get Node.js®",
  "Install Bun v1.4.0" (version pinned in the label), "Get started" →
  installation docs [37], [39], [40].
- **Honest, checkable claims.** Bun pairs performance claims with a
  "reproduce" link to the benchmark harness [39]; Node links the exact
  release versions [37]. No unverifiable superlatives on any fetched page.
- **Nav = Docs + Blog + GitHub + (Install/Download), visible on desktop.**
  Every fetched page exposes its top-level links on desktop; hamburger-style
  toggles are present but mobile-scoped ("aria-label='Open menu'",
  "Toggle navigation menu") [37], [39], [43], [44]. Vercel uses the APG
  disclosure-dropdown model on desktop (`aria-expanded` buttons +
  `aria-controls` panels) [43], [15].
- **Footers are big, grouped, and carry the legal/about links.** 14–84 links,
  always including About, License/Trademark/Legal, and Privacy [37–44] —
  the natural home for epher's planned About and Privacy pages.
- **A "replaces/compares" matrix sells to technical audiences.** Bun's
  feature grid names what each component replaces ("replaces npm · yarn ·
  pnpm") [39]; epher's four-builds card grid (CLI/REPL/TUI/desktop in one
  binary) is the same genre.

---

## 3. WCAG 2.2 AA criteria that bind a marketing site with nav + hamburger

Quoted from the Recommendation [1]; intent from the Understanding docs
[2–13]. epher targets AA; AAA numbers below are included where the task asks
or the Understanding doc recommends aiming at them.

| SC | Level | Requirement (quoted) | What it means for the marketing site |
|---|---|---|---|
| 1.4.1 Use of Color | A | "Color is not used as the only visual means of conveying information, indicating an action, prompting a response, or distinguishing a visual element" [1] | Accent-colored links keep underlines; "current page" in nav gets more than color (e.g. `aria-current` + weight) [12] |
| 1.4.3 Contrast (Minimum) | AA | text ≥ 4.5:1; large-scale text ≥ 3:1; logotypes exempt [1] | All body/CTA/link text incl. muted text and button labels — computed per §5. "Computed values should not be rounded (e.g., 4.499:1 would not meet the 4.5:1 threshold)" [2] |
| 1.4.6 Contrast (Enhanced) | AAA | text ≥ 7:1; large text ≥ 4.5:1 [1] | Not required at AA; the recommended palette happens to clear 7:1 for accent-on-dark (§5) at AAA margins |
| 1.4.8 Visual Presentation | AAA | "Width is no more than 80 characters or glyphs (40 if CJK)"; leading ≥ 1.5; text resizable 200% [1], [13] | The normative ceiling behind the 66ch measure (§1.1); CJK clause applies to the zh-CN locale |
| 1.4.11 Non-text Contrast | AA | "The visual presentation of the following have a contrast ratio of at least 3:1 against adjacent color(s)": UI components/states and graphical objects [1] | Input/card borders, focus rings, icon buttons, the hamburger's 3-line glyph, download-link borders. Canonical pass: grey #767676 border on white [4]. "For people with color vision deficiency … hue and saturation have minimal or no effect on legibility" — pick accent steps by luminance [2] |
| 2.1.1 Keyboard | A | "All functionality of the content is operable through a keyboard interface without requiring specific timings" [1] | Hamburger opens/links/closes/dismisses fully from keyboard; theme toggle and language select keyboard-operable [9] |
| 2.4.1 Bypass Blocks | A | (skip link) | Keep the existing skip link; the WAI menus tutorial lists it first among menu-page techniques [17] |
| 2.4.7 Focus Visible | AA | "Any keyboard operable user interface has a mode of operation where the keyboard focus indicator is visible" [1]; "must not be time limited" [5] | `:focus-visible` ring on every interactive element incl. hamburger and mobile nav links; ring must itself pass 1.4.11's 3:1 ("focus indicators … are also subject to Success Criterion 1.4.11" [5], [4]) |
| 2.4.11 Focus Not Obsscured (Minimum) | AA | "When a user interface component receives keyboard focus, the component is not entirely hidden due to author-created content" [1] | Sticky header must not fully cover the focused element while tabbing; scroll-padding if sticky. Opened menus are fine while non-persistent: "an open dropdown that does not close when no longer focused is not following this convention" [6] |
| 2.5.8 Target Size (Minimum) | AA | target ≥ "24 by 24 CSS pixels", exceptions: Spacing (24px-dia circles don't intersect), Equivalent, Inline, User Agent Control, Essential [1] | Minimum for every nav link, icon button, and footer link. Understanding: "As a best practice it is recommended to at least meet the minimum size requirement … For important links/controls, consider aiming for the stricter 2.5.5" [7] |
| 2.5.5 Target Size (Enhanced) | AAA | ≥ "44 by 44 CSS pixels" (with equivalent/inline/UA/essential exceptions) [1] | epher's existing 44px buttons/selects already meet this best practice [8]; keep 44px for the hamburger and theme toggle |
| 4.1.2 Name, Role, Value | A | name and role programmatically determinable; states programmatically settable/notifiable [1] | Native elements for links/selects/buttons; the hamburger `<button>` gets its name from a text label ("Menu") and exposes state via `aria-expanded`. "Standard HTML controls already meet this success criterion when used according to specification" [10] |
| 2.3.3 Animation from Interactions | AAA | "Motion animation triggered by interaction can be disabled, unless the animation is essential" [1] | Not required at AA, but see §6: implement `prefers-reduced-motion` regardless — cheap and the right side of the intent ("Some users experience distraction or nausea from animated content" [11]) |

Also binding, in the background: 3.1.1 (the existing `lang`/`dir` per locale
must survive the redesign), 1.4.10 Reflow (single column at 320px), 1.4.4
(200% zoom) — unchanged from `docs/accessibility.md`.

---

## 4. Accessible hamburger / disclosure navigation (APG)

The APG models a show/hide nav as the **Disclosure pattern**, not a menu
widget. Direct quotes from the pattern and its navigation examples:

- **Semantics.** "The element that shows and hides the content has role
  button" (a native `<button>` provides this). "When the content is visible,
  the element with role button has `aria-expanded` set to true. When the
  content area is hidden, it is set to false." `aria-controls` is explicitly
  optional: "Optionally, the element with role button has a value specified
  for `aria-controls` that refers to the element that contains all the
  content that is shown or hidden" [14].
- **Not a menu role.** "Although this example uses the word 'menu' in the
  colloquial sense … it does not use the WAI-ARIA menu role … Typical site
  navigation does not need all the keyboard interactions specified by the
  menu and menubar pattern" [15]. (The menu role would obligate arrow-key
  management and focus trapping the site nav doesn't want.)
- **Keyboard.** Enter and Space toggle ("Enter: activates the disclosure
  control and toggles the visibility"; same for Space) [14]. Tab/Shift+Tab
  "move keyboard focus among top-level buttons, and if a dropdown is open,
  into and through links in the dropdown" — the links are ordinary tab stops,
  no focus trap [15]. **Escape:** "If a dropdown is open, closes it and sets
  focus on the button that controls that dropdown" [15]. Optional
  arrows/Home/End "supplement, but do not replace, tabbing among buttons and
  links" [15].
- **Closing on blur.** "If a dropdown is open and focus is inside the
  navigation region, pressing Esc will close the dropdown. Moving focus out
  of the navigation region also closes an open dropdown. Implementing this
  Esc behavior is necessary to meet the WCAG 2.1 1.4.13: Content on Hover or
  Focus criterion" [15]. This non-persistence is also what keeps 2.4.11
  (Focus Not Obscured) satisfied — a dropdown that stayed open after focus
  left is the Understanding doc's named risk case [6], [15].
- **Landmark + list structure.** "The list that contains them is wrapped in
  a navigation landmark named …" and "The semantics of the list structure
  communicates the hierarchy of the navigation system to assistive
  technology users" — `<nav aria-label>` containing a `<ul>` [15]. The
  variant with top-level links alongside disclosure buttons keeps the same
  structure [16].
- **Visual state, styled off ARIA.** "CSS attribute selectors (e.g.
  `[aria-expanded="false"]`) are used to synchronize the visual states with
  the value of the aria-expanded attribute"; the caret indicator is built
  "using CSS ::after pseudo element border styles so the caret is reliably
  rendered in high contrast mode of operating systems and browsers" [15].
  `aria-current="page"` marks the current page [15].
- **Hover behavior.** The WAI menus tutorial's requirement: "In fly-out
  menus, submenus should not disappear immediately after the mouse has left
  the clickable area" [17]; web.dev adds to combine hover with focus and
  never hide important information behind hover alone [24].
- **Don't hide the nav on desktop.** web.dev's UI-patterns guidance: "Try to
  find a strategy that avoids hiding your navigation. If you have a
  relatively small number of items, you can style the navigation to look
  good on small screens"; the overflow pattern (horizontal scroll with a
  truncation gradient) scales further; "As a last resort you could choose to
  have your navigation hidden by default and provide a toggle mechanism …
  This is called progressive disclosure. Make sure the button that toggles
  the display of the navigation is labeled. Don't rely on an icon to be
  understood. An unlabelled icon is 'mystery meat' navigation" [23]. Field
  survey agrees: all eight fetched sites keep top-level nav visible on
  desktop and scope the hamburger toggle to narrow viewports [37–44]. For
  epher with ~5 links (Guide, About, Privacy, Source, theme, language): show
  links inline at desktop widths, collapse to a labeled disclosure button
  only under the breakpoint where they stop fitting.

**Recommended hamburger recipe (synthesis of [14–16], [23]):** a real
`<button>` with visible text "Menu" (not icon-only), `aria-expanded` synced
with `[aria-expanded]` CSS attribute selectors, `aria-controls` optional but
harmless, `aria-hidden="true"` SVG decoration; menu is a `<nav>` landmark
containing a `<ul>` of links, each ≥ 44×44px target; Enter/Space toggle;
Escape closes and returns focus to the button; focus leaving the nav closes
it; current page marked with `aria-current="page"` plus non-color styling;
no `menu` role, no focus trap, no hover-open on desktop.

---

## 5. Accent color selection (computed)

**Method.** WCAG relative luminance (sRGB channels linearized at the
0.03928 threshold, then L = 0.2126·R′ + 0.7152·G′ + 0.0722·B′) and contrast
ratio (L1 + 0.05)/(L2 + 0.05), per the spec's definition [1] and the
Understanding doc's precision note [2]. Worked example for the dark
background `#141416` (R=G=0x14=20, B=0x16=22):

```
R' = ((20/255 + 0.055)/1.055)^2.4 = 0.006995   G' = 0.006995
B' = ((22/255 + 0.055)/1.055)^2.4 = 0.008023
L(bg) = 0.2126·0.006995 + 0.7152·0.006995 + 0.0722·0.008023 = 0.007070
```

For `#2dd4bf` (teal-400): R′=0.026241, G′=0.658375, B′=0.520996 →
L = 0.514064, so ratio = (0.514064 + 0.05)/(0.007070 + 0.05) = **9.88:1**.

**Candidates on the dark background (text threshold 4.5:1; UI/border
threshold 3:1):**

| Candidate | L | vs `#141416` | Passes 4.5:1 text? |
|---|---|---|---|
| `#2dd4bf` teal-400 | 0.514064 | **9.88:1** | yes |
| `#5eead4` teal-300 | 0.659788 | **12.44:1** | yes |
| `#22d3ee` cyan-400 | — | **10.18:1** | yes |
| `#a3e635` lime-400 | — | **12.20:1** | yes |
| `#34d399` emerald-400 | — | **9.57:1** | yes |
| `#ff9f0a` amber (current, reference) | — | 8.95:1 | yes |

Every candidate clears 4.5:1 on near-black — with luminance this high the
differentiator is identity and light-mode pairing, not dark-mode contrast.
The same holds on the current `#1c1c1e` dark theme (teal-400 = 9.14:1,
teal-300 = 11.50:1) and on panel surfaces `#1f1f23` (8.82:1 / 11.10:1) and
`#2c2c2e` (7.49:1 / 9.42:1) — computed, so the accent survives any plausible
dark surface.

**Light-mode counterparts on `#ffffff` (text threshold 4.5:1):**

| Family | Dark-mode accent | Light-mode steps (ratio on white) | Text-safe? |
|---|---|---|---|
| teal | `#2dd4bf` 9.88:1 | 500 `#14b8a6` 2.49:1 ✗ · 600 `#0d9488` 3.74:1 ✗ · **700 `#0f766e` 5.47:1 ✓** · 800 `#115e59` 7.58:1 ✓ | ✓ at 700 |
| cyan | `#22d3ee` 10.18:1 | 600 `#0891b2` 3.68:1 ✗ · **700 `#0e7490` 5.36:1 ✓** · 800 `#155e75` 7.27:1 ✓ | ✓ at 700 |
| lime | `#a3e635` 12.20:1 | 600 `#65a30d` 3.09:1 ✗ · **700 `#4d7c0f` 4.99:1 ✓** · 800 `#3f6212` 7.08:1 ✓ | ✓ at 700 |
| emerald | `#34d399` 9.57:1 | 600 `#059669` 3.77:1 ✗ · **700 `#047857` 5.48:1 ✓** · 800 `#065f46` 7.68:1 ✓ | ✓ at 700 |

Note the pattern: **the 400-step used on dark and the 600-step used in many
palette tools both fail on white**; every family needs its 700-step for
light-mode text. Button text compounds this: white on `#0d9488` is only
3.74:1 (fails), white on `#0f766e` is 5.47:1 (passes); dark `#141416` on
`#2dd4bf` is 9.88:1 (passes) and on `#5eead4` is 12.44:1 (passes).

**Recommendation — teal, one family, three working tokens:**

- Dark theme: **accent `#2dd4bf`** (9.88:1 on `#141416`; 9.14:1 on current
  `#1c1c1e`), **hover `#5eead4`** (12.44:1). Button: fill `#2dd4bf`, label
  `#141416` (9.88:1). Focus ring `#2dd4bf`: 9.88:1 vs bg, 8.82:1 vs a
  `#1f1f23` panel — far above the 3:1 of 1.4.11 [4].
- Light theme: **accent text/links `#0f766e`** (5.47:1 on white),
  **hover `#115e59`** (7.58:1). Button: fill `#0f766e`, label white
  (5.47:1).
- Why teal over the passing alternatives: cyan-400 is numerically fine but
  the cyan/blue family is the default-blue-adjacent look the redesign is
  avoiding and is heavily used by AI products (field observation [42–44]);
  lime and emerald are green-family — they collide with success/error
  semantics and sit on the red-green confusion axes for protan/deutan users,
  which matters for epher's *product* UI (errors, graph curves) sharing the
  accent hue (analysis). Contrast-wise all four families comply; only teal
  avoids both clichés.
- **Desaturation and color-blind pitfalls:** pick and verify steps by
  *luminance*, not hue — "For people with color vision deficiency who are
  not able to distinguish certain shades of color, hue and saturation have
  minimal or no effect on legibility as assessed by reading performance"
  [2]; and never encode meaning by the accent alone (1.4.1 [1], [12]).
  Carbon's rule: "Don't rely on color alone to convey meaning … includes
  conveying information, indicating an action, prompting the user for a
  response, or distinguishing one visual element from another" [36].

---

## 6. Reduced motion

- **WCAG 2.3.3 Animation from Interactions (AAA):** "Motion animation
  triggered by interaction can be disabled, unless the animation is
  essential to the functionality or the information being conveyed" [1].
  epher commits to AA, so this is not strictly required — but the intent is
  health, not compliance: "if scrolling a page causes elements to move
  (other than the essential movement associated with scrolling) it can
  trigger vestibular disorders. Vestibular (inner ear) disorder reactions
  include dizziness, nausea and headaches"; parallax is the named frequent
  offender, and the remedies listed are exactly: avoid unnecessary
  animation, provide an off control, or "take advantage of the reduce motion
  feature in the user agent or operating system" [11]. The doc also draws
  the boundary with 2.2.2: interaction-triggered animation is 2.3.3's domain
  while page-initiated (>5s) animation is 2.2.2's — a scroll reveal is
  interaction-triggered [11].
- **`prefers-reduced-motion` media query.** MDN documents the two values —
  `no-preference` ("evaluates as false in the boolean context") and `reduce`
  ("interfaces should minimize movement or animation, preferably to the
  point where all non-essential movement is removed") — and warns that
  "Animations such as scaling or panning large objects can be vestibular
  motion triggers", while a "dissolve animation … is a more muted animation
  that is not a vestibular motion trigger" [18].
- **The web.dev pattern.** Scope animations to the affirmative query —
  animate only under `(prefers-reduced-motion: no-preference)`, so opted-out
  users *and* browsers that don't support the query get the static version —
  and optionally lazy-load the animation CSS with
  `<link rel="stylesheet" href="animations.css" media="(prefers-reduced-motion: no-preference)">`
  [28]. For JS-driven animation (e.g. IntersectionObserver reveals), listen
  for changes: "While CSS rules will be dynamically triggered by the browser
  when the user preference changes, for JavaScript animations I have to
  listen for changes myself, and then manually stop my potentially in-flight
  animations" [28].
- **Rules for the epher redesign:** (a) keep the existing global
  `@media (prefers-reduced-motion: reduce)` kill-switch in `styles.css`
  (duration 0.01ms etc.) — it matches this guidance; (b) make scroll-reveal
  content fully visible before/without JS (reveal is enhancement, not
  content gating — consistent with web.dev's "don't hide" principle [23]);
  (c) animate only `opacity`/`transform`, never size-inducing properties;
  (d) keep `scroll-behavior: auto` as today; (e) micro-interactions in the
  70–240ms Carbon range need no gating beyond the global kill-switch [33];
  (f) no parallax, no autoplaying hero motion (2.2.2 [1], [11]).

---

## 7. Synthesis — what the redesign should do

(Analysis; sources in brackets.)

1. **Adopt the teal tokens** of §5 unchanged, and record the computed
   ratios in `site/styles.css` comments + `docs/accessibility.md` exactly as
   the current amber ones are recorded (house convention).
2. **Keep the system font stack** with the Noto per-script fallbacks [20],
   [22]; no webfont download for the marketing page.
3. **Fix the hero clamp middle term** to include a rem component
   (`clamp(2.5rem, 1.5rem + 5vw, 4.25rem)`-style) so zoom participates [22].
4. **Constrain prose measure to ~66ch** (`max-inline-size: 66ch` on lede/
   guide-ish paragraphs); card grids keep
   `repeat(auto-fit, minmax(250px, 1fr))` [22], [25].
5. **Adopt the 8px mini-unit spacing ladder** (2/4/8/12/16/24/32/48/64/96)
   for section rhythm, replacing ad-hoc paddings [32], [35].
6. **Nav:** desktop shows all top-level links (Guide, About, Privacy,
   Source, theme, language) — only collapse behind a labeled "Menu"
   disclosure button when they stop fitting [23]; implement per §4 (native
   button, `aria-expanded`, Escape-closes-and-refocuses, focus-out closes,
   `aria-current="page"`, 44px targets) [14–16].
7. **Add About and Privacy as footer pages** (field convention: about/legal
   links live in a multi-column footer) [37–44]; they inherit the same
   tokens, measure, and criteria table above.
8. **Hero:** one-line value prop + single concrete CTA ("Get epher" →
   downloads, matching the field survey's "Get Node.js®"/"Install Bun"
   pattern) + a real code example (input → result) as the hero art [37–40].
9. **Motion budget:** hover/press states 70–150ms, panel/menu expansion
   150–240ms, nothing else; all gated by the global reduced-motion rule
   [28], [33].
10. **Surfaces:** panels by layer tokens with ≥3:1 borders (keep
    `#76767a`-class grey, the Understanding doc's own passing example [4]);
    shadows optional and subtle; any hero gradient must be validated for
    text contrast at its worst point [36].

---

## Summary of concrete recommendations

| Token / decision | Value | Source or computation |
|---|---|---|
| Font stack | system stack + `Noto Sans`, `Noto Sans Arabic`, `Noto Sans Devanagari` (keep) | MDN system-ui caveat + web.dev perf [20], [22] |
| Body type | `1rem`, `line-height: 1.5–1.65` | MDN 1.5–2 [23m]; web.dev 1.65@66ch [22] |
| Display type | `clamp(2.5rem, 1.5rem + 5vw, 4.25rem)` | web.dev clamp pattern [22], MDN clamp [19] |
| Type scale steps | 0.875 / 1 / 1.25 / 1.5 / 2.25 / display | subset of Carbon 12–92px ladder [31] |
| Measure (prose) | `max-inline-size: 66ch` (≤80ch hard ceiling) | web.dev/Bringhurst 45–75ch [22]; WCAG 1.4.8 ≤80ch [13] |
| Card grid | `repeat(auto-fit, minmax(250px, 1fr))` | web.dev auto-fill/minmax idiom [25] |
| Spacing scale | 8px mini unit; tokens 2,4,8,12,16,24,32,48,64,96 | Carbon 2x Grid + spacing tokens [32], [35] |
| Dark bg | `#141416` (L = 0.007070) or keep `#1c1c1e` | computed §5 |
| `--accent` (dark) | `#2dd4bf` teal-400 — **9.88:1** on `#141416` (9.14:1 on `#1c1c1e`) | computed §5; needs ≥4.5:1 [1], [2] |
| `--accent` hover (dark) | `#5eead4` teal-300 — **12.44:1** | computed §5 |
| Button dark | fill `#2dd4bf`, label `#141416` — **9.88:1** | computed §5 |
| `--accent` (light text/links) | `#0f766e` teal-700 — **5.47:1** on white | computed §5 |
| `--accent` hover (light) | `#115e59` teal-800 — **7.58:1** | computed §5 |
| Button light | fill `#0f766e`, label white — **5.47:1** | computed §5 |
| Focus ring | `#2dd4bf` — 9.88:1 vs bg, 8.82:1 vs `#1f1f23` panel (≥3:1 required) | computed §5; 1.4.11 + 2.4.7 [4], [5] |
| Border grey | keep `#76767a`-class (3:1+ on both themes) | Understanding 1.4.11 example #767676 [4]; house audit |
| Nav pattern | desktop: visible links; mobile: labeled "Menu" `<button>`, `aria-expanded`, Escape closes + refocus, focus-out closes, `aria-current`, no menu role, no trap | APG disclosure + nav examples [14–16]; web.dev [23] |
| Target sizes | ≥ 44×44 for nav/buttons/selects (24×24 AA floor) | 2.5.8 AA [1], [7]; 2.5.5 AAA best practice [1], [8] |
| Motion durations | 70–150ms hover/press, 150–240ms panels | Carbon duration tokens [33] |
| Reduced motion | global `prefers-reduced-motion: reduce` kill-switch; reveal content visible without JS; animate opacity/transform only; no parallax/autoplay | MDN [18]; web.dev [28]; WCAG 2.3.3/2.2.2 [1], [11] |
| New pages | About + Privacy, footer-linked, same tokens/criteria | field survey [37–44] |

---

## Citations

All sources accessed 2026-08-20. WCAG quotes are from the W3C Recommendation
and its WAI Understanding documents (w3.org/WAI/WCAG22/Understanding/).
APG quotes from the WAI-ARIA Authoring Practices Guide. web.dev sources are
Google's Learn Responsive Design course lessons and articles. Carbon sources
are IBM's design system, verified against the source markdown in the
carbon-website repo. Field-survey pages were fetched live and parsed; code/
footer counts refer to the served HTML.

**W3C / WAI (WCAG 2.2 + Understanding + APG + tutorial)**

1. *Web Content Accessibility Guidelines (WCAG) 2.2* (W3C Recommendation; normative SC text) — https://www.w3.org/TR/WCAG22/
2. *Understanding SC 1.4.3 Contrast (Minimum)* — https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html
3. *Understanding SC 1.4.6 Contrast (Enhanced)* — https://www.w3.org/WAI/WCAG22/Understanding/contrast-enhanced.html
4. *Understanding SC 1.4.11 Non-text Contrast* — https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html
5. *Understanding SC 2.4.7 Focus Visible* — https://www.w3.org/WAI/WCAG22/Understanding/focus-visible.html
6. *Understanding SC 2.4.11 Focus Not Obscured (Minimum)* — https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html
7. *Understanding SC 2.5.8 Target Size (Minimum)* — https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
8. *Understanding SC 2.5.5 Target Size (Enhanced)* — https://www.w3.org/WAI/WCAG22/Understanding/target-size-enhanced.html
9. *Understanding SC 2.1.1 Keyboard* — https://www.w3.org/WAI/WCAG22/Understanding/keyboard.html
10. *Understanding SC 4.1.2 Name, Role, Value* — https://www.w3.org/WAI/WCAG22/Understanding/name-role-value.html
11. *Understanding SC 2.3.3 Animation from Interactions* — https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html
12. *Understanding SC 1.4.1 Use of Color* — https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html
13. *Understanding SC 1.4.8 Visual Presentation* — https://www.w3.org/WAI/WCAG22/Understanding/visual-presentation.html
14. *APG: Disclosure (Show/Hide) Pattern* — https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/
15. *APG Example: Disclosure Navigation Menu* — https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/examples/disclosure-navigation/
16. *APG Example: Disclosure Navigation Menu with Top-Level Links* — https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/examples/disclosure-navigation-hybrid/
17. *WAI Tutorial: Menus* — https://www.w3.org/WAI/tutorials/menus/

**MDN**

18. *prefers-reduced-motion* — https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion
19. *clamp()* — https://developer.mozilla.org/en-US/docs/Web/CSS/clamp
20. *font-family* (system-ui note) — https://developer.mozilla.org/en-US/docs/Web/CSS/font-family
21. *oklch()* — https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/oklch
23m. *CSS styling text fundamentals* (line-height 1.5–2) — https://developer.mozilla.org/en-US/docs/Learn/CSS/Styling_text/Fundamentals

**web.dev**

22. *Learn Responsive Design: Typography* — https://web.dev/learn/design/typography
23. *Learn Responsive Design: User interface patterns* — https://web.dev/learn/design/ui-patterns
24. *Learn Responsive Design: Interaction* — https://web.dev/learn/design/interaction
25. *Learn Responsive Design: Macro layouts* — https://web.dev/learn/design/macro-layouts
26. *Learn Responsive Design: Micro layouts* — https://web.dev/learn/design/micro-layouts
27. *Learn Responsive Design: Theming* — https://web.dev/learn/design/theming
28. *prefers-reduced-motion: Hello darkness, my old friend* — https://web.dev/articles/prefers-reduced-motion
29. *prefers-color-scheme: Hello darkness, my old friend* — https://web.dev/articles/prefers-color-scheme

**IBM Carbon design system** (published pages; source markdown verified at
github.com/carbon-design-system/carbon-website, `src/pages/elements/…`)

31. *Typography overview* (type scale, productive/expressive) — https://carbondesignsystem.com/elements/typography/overview/
32. *Spacing overview* (token ladder) — https://carbondesignsystem.com/elements/spacing/overview/
33. *Motion overview* (duration tokens, productive/expressive) — https://carbondesignsystem.com/elements/motion/overview/
34. *Color overview / usage* (one action family, layer tokens) — https://carbondesignsystem.com/elements/color/overview/
35. *2x Grid overview* (8px mini unit) — https://carbondesignsystem.com/elements/2x-grid/overview/
36. *Accessibility: color* (3:1 boundaries, color-blindness, gradient text) — https://carbondesignsystem.com/guidelines/accessibility/color/

**Field survey** (fetched live 2026-08-20; structure signals parsed from served HTML)

37. Node.js — https://nodejs.org/en
38. Deno — https://deno.com
39. Bun — https://bun.sh
40. Tailwind CSS — https://tailwindcss.com
41. Rust — https://rust-lang.org
42. Stripe — https://stripe.com
43. Vercel — https://vercel.com
44. Linear — https://linear.app
