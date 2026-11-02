# Accessibility pages in the wild: what W3C and GOV.UK prescribe, what large and small sites actually publish; and what that means for an epher.org accessibility page

Research of 2026-11-02 (branch `staging`, uncommitted), ahead of adding an
"Accessibility" menu item to epher.org (slotted before "Privacy", matching the
current nav order Examples / Scripts / Language reference / Features / IDE
Extension / Privacy / Source code; nav read for context only, `site/index.html`
line 92). The headline: **accessibility pages come in two genres that barely
overlap**; formal *statements* (scope, conformance claim + WCAG level, known
limitations with workarounds, feedback channel, date) and *feature/marketing*
pages (features grouped by disability plus support links); and **the structural
sources agree on a small core**: W3C WAI says three things are the required
minimum (commitment, standard applied, contact information) and five more are
advisable; GOV.UK's legally-required format adds enforcement wording, review
dates, and WCAG-success-criterion-level disclosure of failures. In the small/FOSS
genre the norm is **having no page at all** (Signal, Bitwarden, GNOME,
Thunderbird, Mastodon, Ghost, Element, VLC, LibreOffice, Mozilla all checked;
absent); the credible minimum among those that do have one is Discourse's
four-sentence statement, and WordPress shows the honest "AA where possible +
known-issues tracker" pattern. Even Wikipedia has no statement about its own
site: `Wikipedia:Accessibility` is a Manual-of-Style guideline, and MediaWiki's
accessibility page is now a stub.

## Goal

Decide what an epher.org accessibility page should actually contain by looking at
(a) the two structural primary sources; W3C WAI's statement guidance/generator
and GOV.UK's legally-required format, and (b) real pages across three genres;
conformance statements, feature/marketing pages, and small/FOSS projects nearest
epher's situation. Observations only; nothing on the site was touched and no
page copy was drafted.

## Method (all claims verified)

Everything below was fetched directly on 2026-11-02 with `curl -sL` and read as
extracted text, unless marked UNVERIFIED:

- **Primary sources**: w3.org/WAI/planning/statements/ (Quick Start) and its
  generator tool page; both GOV.UK publications (the sample-statement
  publication page and the full sample statement attachment) and GOV.UK's own
  live statement [1]–[6].
- **Genre A**: NHS website statement, BBC accessibility hub [7][8].
- **Genre B**: apple.com/accessibility, microsoft.com/accessibility,
  google.com/accessibility; all server-rendered enough for curl; quotes taken
  from the fetched HTML [9]–[11].
- **Genre C**: discourse.org/accessibility, wordpress.org/accessibility,
  W3C WAI's own statement, en.wikipedia.org/wiki/Wikipedia:Accessibility,
  mediawiki.org/wiki/Accessibility, foundation.wikimedia.org/wiki/Policy:Accessibility,
  support.mozilla.org Firefox accessibility KB [3][12]–[18].
- **Absence survey** (probed live, HTTP status checked): signal.org,
  bitwarden.com (three candidate paths + homepage link scan), gnome.org,
  thunderbird.net, joinmastodon.org, ghost.org, element.io, videolan.org,
  libreoffice.org, mozilla.org [19]–[22].
- **Wayback Machine fallback** where live pages were missing or JS shells:
  web.archive.org CDX API for mozilla.org/en-US/accessibility (only capture is
  a 404, 2025-08-18), accessibility.gnome.org (zero captures), and the Firefox
  accessibility KB article (full snapshot 2022-11-08, since the live SUMO page
  now serves a 3 KB JS shell) [17][18][21].

## The structural primary sources

### W3C WAI "Developing an Accessibility Statement"; the required/advisable core [1]

The W3C WAI Quick Start page gives the smallest authoritative checklist. In
full, verbatim:

> Accessibility statements should contain at least the following:
>
> - A commitment to accessibility for people with disabilities
> - The accessibility standard applied, such as WCAG 2.2
> - Contact information in case users encounter problems
>
> It is also advisable to include the following information:
>
> - Any known limitations, to avoid frustration of your users
> - Measures taken by your organization to ensure accessibility
> - Technical prerequisites, such as supported web browsers
> - Environments in which the content has been tested to work
> - References to applicable national or local laws and policies

On audience and tone:

> "Accessibility statements are primarily for users of your content. … It is
> important to write in simple language, and to provide information that is
> useful to the users, rather than use the language of developers and lawyers."

> "In particular, accessibility statements should explain functionality and
> known limitations in common terms. For example, rather than to say 'WCAG
> Success Criterion 1.2.2 was not met', it is better to say 'videos do not have
> captions'."

On placement; relevant to epher's planned menu slot:

> "Accessibility statements should be easy to find. Linking them from several
> places, such as from the footer, help menu, sitemap, about page, and other
> prominent areas helps users to find them. Use consistent link names for your
> accessibility statements to help users recognize them."

Notable: GOV.UK's sample, by contrast, *requires* the SC numbers in the
regulated genre ("explain which of the WCAG 2.1 AA success criteria the problems
fail on"; updated to 2.2), so the plain-language rule is a WAI recommendation,
not a universal law; the two sources coexist because they serve different
genres [1][5].

### The WAI generator tool; the full section skeleton [2]

The generator (reachable, static HTML with the whole form) says: "None of the
fields are required." Its form groups, in order: **Basic information**
(organization, site/app name + address; "For mobile applications, include
version information and the release date"; accessibility standard followed;
default "WCAG 2.2 level AA"; conformance status; additional accessibility
considerations; feedback options; publication date), **Your efforts**
(organizational measures), **Technical information** (accessibility limitations,
compatibility with user environment, known incompatibility, technologies used),
and **Approval and complaints process** (formal complaints, formal approval).

Key field semantics worth borrowing:

- Conformance status is a four-way choice: "Fully conformant / Partially
  conformant / Non conformant / Not assessed".
- Feedback: "Ideally you should provide more than one option. Also indicate the
  duration after which users can expect a response from your organization."
- Each limitation is a five-part record: content part / description of the issue
  / why the issue occurs / "What we are doing about it" / "What to do in the
  meantime".
- Compatibility: list tested environments ("browser X with assistive technology
  Y on operating system Z") *and* known incompatibilities.
- On the statement date: "Ideally the date of an accessibility statement should
  not exceed one year, or it may be considered unmaintained. Writing out the
  month makes the date clearer internationally (eg. '1 February 2019' rather
  than '01/02/2019', which is ambiguous)."

The generated statement's section headings: intro line ("This is an
accessibility statement from …"), Measures to support accessibility, Conformance
status, Additional accessibility considerations, Feedback, Compatibility with
browsers and assistive technology, Technical specifications, Limitations and
alternatives, Assessment approach (self-/external evaluation), Evaluation
report/statement, Formal complaints, Formal approval, Date.

### GOV.UK: the UK legally-required format [4][5]

The publication page ("Sample accessibility statement for a fictional public
sector website or app", GDS/CDDO, first published 17 May 2019, last updated 30
September 2024) says: "Some of the wording is legally required, so make sure you
include that in your statement", and the sample marks the fixed wording
explicitly: "This wording is legally required and must not be changed." It was
"Updated 2 April 2020 … in line with the EU model statement"; i.e. the UK
format descends from the EU Web Accessibility Directive model [4].

The mandatory skeleton of the sample statement (attachment), in order [5]:

1. **Scope**; "This accessibility statement applies to [scope of statement…
   ]" (note: one statement may cover multiple domains; "For mobile apps also
   include the application's version information and publication date").
2. **"How you should be able to use this website"**; the capability list
   (colours/contrast/fonts via browser settings; "zoom in up to 400% without the
   text spilling off the screen"; "navigate most of the website using a keyboard
   or speech recognition software"; "listen to most of the website using a
   screen reader (including the most recent versions of JAWS, NVDA and
   VoiceOver)"), plus plain-text commitment and an AbilityNet pointer.
3. **"How accessible this website is"**; known issues "in order of most impact
   to least impact".
4. **"Feedback and contact information"**; who to contact, alternative-format
   requests, "We'll consider your request and get back to you in [number] days."
5. **"Enforcement procedure"**; fixed EHRC/EASS wording.
6. **"Technical information about this website's accessibility"**; commitment
   "in accordance with the Public Sector Bodies (Websites and Mobile
   Applications) (No. 2) Accessibility Regulations 2018".
7. **"Compliance status"**; "The website has been tested against the Web
   Content Accessibility Guidelines (WCAG) 2.2 AA standard," then one of three
   legally fixed sentences: "(a) This website is fully compliant… (b) This
   website is partially compliant… due to the non-compliances/exemptions… (c)
   This website is not compliant…"; "There's a legally required way of
   expressing the compliance status of your website, so do not change it."
8. **"Non-accessible content"**; heading and the sentence "The content listed
   below is non-accessible for the following reasons." are fixed; subsections
   "Non-compliance with the accessibility regulations" (each problem + WCAG SC
   + fix date), "Disproportionate burden", "Content that's not within the scope
   of the accessibility regulations" are all legally required subheadings.
9. **"What we're doing to improve accessibility"**; optional roadmap.
10. **"Preparation of this accessibility statement"**; "This statement was
    prepared on [date]. It was last reviewed on [date]." / "This website was
    last tested on [date] against the WCAG 2.2 AA standard." / who tested /
    optional test-report link. "The statement must be reviewed at least once a
    year, even if there have not been significant changes to the website."

So the mandatory elements are: scope, compliance status with WCAG level claimed,
non-accessible content + reasons/workarounds, feedback contact, enforcement,
preparation/review dates, and test date/method.

### GOV.UK's own statement; the format in production [6]

https://www.gov.uk/help/accessibility-statement follows the sample exactly, and
shows the genre's characteristic honesty at scale: **"This website is partially
compliant with the Web Content Accessibility Guidelines version 2.2 AA
standard, due to the non-compliances and exemptions listed below."** It then
enumerates ~20 specific failures, each tied to a success criterion ("Some tables
do not have table row or column headers. … This fails WCAG 2.2 success criterion
1.3.1 (Info and Relationships)."). Maintenance dates: "This statement was
prepared on 23 September 2019. It was last reviewed on 29 January 2026." and
"This website was last tested in March 2025 against the WCAG 2.2 AA standard.
This test of a representative sample of pages was carried out by the Digital
Accessibility Centre (DAC)."

## Genre A: conformance statements in the wild

### NHS website; GOV.UK format, health-sector edition [7]

- URL: https://www.nhs.uk/accessibility/ (the /nhs-website/accessibility/ path
  404s; the statement lives at the short path).
- Genre: statement (regulated).
- Sections: capability list → how accessible this website is → PDFs → other
  non-accessible parts → feedback and contact → alternative formats → reporting
  problems → enforcement procedure (EHRC/EASS) → technical information →
  compliance status → non-accessible content (forms / PDFs / 3rd-party tools /
  screen readers…) → out-of-scope content → improvement commitments ("all videos
  to be made AA standard when they are reviewed…", "a principle to move away
  from new content being produced in PDF") → review promise.
- Quote: "This website is partially compliant with the Web Content Accessibility
  Guidelines (WCAG) version 2.2 AA standard, due to the non-compliances listed
  below." Maintenance: "This statement was updated in May 2025."
- Notable: same skeleton as GOV.UK's own, proving the format clones cleanly
  outside GDS; known issues are grouped by content type with per-issue
  explanations.

### BBC; a hub, not a statement [8]

- URL: https://www.bbc.co.uk/accessibility/
- Genre: hybrid hub (help centre index); no conformance claim anywhere on the
  page.
- Sections: intro ("All audiences are important to the BBC.") → BBC
  Accessibility Help (per-product links: iPlayer Accessibility FAQs, Sounds
  accessibility information, audio-described/signed content) → BBC Accessibility
  Policy → Programmes and Accessibility → Design and Technical Resources (GEL
  designing-for-accessibility pages, Mobile & Web Accessibility Guidelines,
  Subtitle Guidelines) → Diversity and Accessibility → Accessibility for
  Suppliers → External Accessibility Help (links out to Apple/Google/Microsoft
  support).
- Quote: "This page provides links to resources and documents to help users
  access the BBC's content and to find out about the BBC approach to the
  development of rich accessible products and services."
- Notable: a large broadcaster treats "accessibility" as a *portal*; the
  statement-like detail is pushed down to per-product pages (which were not
  fetched; UNVERIFIED whether they contain conformance claims).

## Genre B: feature/marketing pages

All three are product marketing with support resources; none of the three
landing pages carries a WCAG conformance claim, a known-issues list, or a
statement date.

### Apple [9]

- URL: https://www.apple.com/accessibility/
- Genre: features/marketing.
- Sections: hero ("Innovation that's accessible by design.") → feature spotlights
  grouped by need ("Must-know vision, hearing, speech, mobility, and cognitive
  features": VoiceOver, Generated Subtitles, AirPods Pro Hearing Health,
  Personal Voice, Voice Control, Accessibility Reader) → resources (Support
  Videos, Accessibility Support, Accessories, third-party Assistive
  Technologies, Older Adults, Workplace, Education, Developer Guides) →
  values cross-links.
- Quote: "The best technology is designed with everyone in mind. That's why our
  products and services have built-in features to help you create, connect, and
  do what you love, your way."
- Notable: the by-disability taxonomy (vision / hearing / speech / mobility /
  cognitive) is the genre's signature; everything is a promise, nothing is a
  limitation.

### Microsoft [10]

- URL: https://www.microsoft.com/accessibility
- Genre: features/marketing + support routing.
- Sections: AI-assisted help → **Disability Answer Desk** ("Where customers with
  disabilities can easily get support with Microsoft 365, Windows, Xbox, and
  Surface. Get support via chat, phone, or ASL") → self-help by product and;
  the key pattern; **by disability**: Vision, Hearing, Neurodiversity, Learning,
  Mobility, Mental health → AI training → news (Ability Summit 2026, Chief
  Accessibility Officer, "Microsoft's commitment to the European Accessibility
  Act").
- Quote: "Get support via chat, phone, or ASL"; support-channel provision is
  the page's real centre of gravity.
- Notable: the only regulatory nod is EAA-related news; no conformance status on
  the page itself.

### Google [11]

- URL: https://www.google.com/accessibility/
- Genre: features/marketing, but with a formal bridge.
- Sections: mission hero → What's New (YouTube series) → newsletters →
  participate (Disability Support team, Build for All Panel, Accessibility User
  Research Panel) → resources → commitment statement.
- Quote: "Google Accessibility Conformance Reports; To make it easier for
  companies, educational institutions, and government agencies to verify
  alignment with accessibility standards, we provide transparent reports about
  how our products currently work for people with disabilities. We offer the
  following English-language Accessibility Conformance Reports (ACR) based on
  the ITI Voluntary Product Accessibility Template (VPAT®)."
- Notable: even Google keeps conformance documentation *off* the marketing page,
  in separate VPAT-based reports; and its commitment names the EAA: "Google is
  committed to building more accessible and useful products and services,
  including by meeting the requirements of the European Accessibility Act (EAA)."

## Genre C: small/FOSS projects; what projects like epher actually say

### Discourse; the credible minimum for a FOSS product [12]

- URL: https://www.discourse.org/accessibility
- Genre: statement (minimal).
- Sections: mission sentence → approach (WCAG + audits) → single feedback
  channel with what-to-include guidance. That's the whole page; four sentences.
- Quotes: "The team building Discourse has set out to create discussion software
  that is free, open source, and accessible to all." / "To guide our ongoing
  effort to build accessible software we follow the W3C's Web Content
  Accessibility Guidelines (WCAG). Discourse has also undergone accessibility
  audits from external experts and automated accessibility tools." / "If you
  find a problem that makes it difficult for you to use Discourse, send an email
  to accessibility@discourse.org. When describing the problem please include
  details about your web browser and any accessibility software you are using."
- Notable: no conformance claim, no date, no limitations list; yet it does the
  three WAI-required things (commitment, standard, contact) plus the Discourse
  touch of telling users *what details to send*. Closest match to epher's
  situation among the positives.

### WordPress; the honest "AA where possible" pattern [13]

- URL: https://wordpress.org/accessibility/
- Genre: statement (FOSS project page, on the product's own site).
- Sections: commitment → conformance aim → known-issues pointer → themes
  caveat → Accessibility Team → handbook → reporting → **ATAG Statement**.
- Quotes: "WordPress aims to make the WordPress Admin and bundled themes fully
  compliant with Web Content Accessibility Guidelines (WCAG) 2.2 AA, where
  possible." / "Some current features and functionality in development may not
  yet fully comply, and known issues are listed in the WordPress Trac
  'accessibility' focus." / "WordPress is not currently conforming with ATAG
  2.0, but pledges to pursue features that are accessible…"
- Notable: instead of maintaining a limitations list in the page, it links to
  the issue tracker's accessibility tag; a natural pattern for a FOSS project
  that already has a public tracker; and it states non-conformance plainly
  (ATAG) rather than staying silent.

### W3C WAI's own statement; a small team using its own tool [3]

- URL: https://www.w3.org/WAI/about/accessibility-statement/ (linked from the
  Quick Start as a generator-made example).
- Genre: statement (complete, generator-shaped, still compact).
- Sections: Summary, Measures to support accessibility, Conformance status,
  Feedback, Compatibility with browsers and assistive technology, Technical
  specifications, Limitations and alternatives, Assessment approach, Formal
  approval.
- Quotes: "Web Accessibility Initiative (WAI) website content posted since May
  2018 fully conforms to WCAG 2.1 Level AA. It partially conforms to Level
  AAA." / "We usually respond to accessibility feedback from e-mail within 3-5
  business days. Sometimes it can take up to 3 weeks. If you do not receive a
  reply to e-mail within 10 business days, you can contact Shawn Henry at
  +1-617-395-7664." / "Several videos on the WAI website use YouTube. As an
  alternative, the videos are provided as MP4 files on a W3C server." / "WAI
  assessed the accessibility of the WAI website by self-evaluation."
- Notable: the best "small organization" model; every limitation comes with an
  *alternative*, response-time honesty includes an escalation contact, and
  "self-evaluation" is presented as a legitimate assessment approach.

### Wikipedia / MediaWiki / Wikimedia Foundation; no site statement at all [14][15][16]

- https://en.wikipedia.org/wiki/Wikipedia:Accessibility redirects to
  **Wikipedia:Manual of Style/Accessibility**; a *content* guideline for
  editors ("Wikipedia pages should have clear formatting, descriptive links, and
  alternative text so that content is easy to navigate and accessible for all
  readers, including those with disabilities."), not a statement about
  Wikipedia's own interface. The task brief's memory that en.wikipedia.org "has
  one via the MediaWiki manual" is out of date.
- https://www.mediawiki.org/wiki/Accessibility is now a two-paragraph stub:
  "MediaWiki accessibility is the goal of making MediaWiki sites easier to
  navigate and read…" plus pointers to the Phabricator `Accessibility` tag. No
  conformance claim, no statement sections.
- https://foundation.wikimedia.org/wiki/Policy:Accessibility returns "There is
  currently no text in this page."; the Foundation's old accessibility policy
  page is empty.
- Notable: the world's largest content platform publishes no accessibility
  statement about itself; its accessibility effort is expressed as *editorial
  guidance* + a bug-tag. For epher, the analogue is docs/accessibility.md (the
  living audit); public but repo-side, not user-facing.

### Firefox/Mozilla; statement page gone; features live in a support KB [17][18]

- https://www.mozilla.org/en-US/accessibility/ → 404 (live, 2026-11-02; the
  only Wayback capture, 2025-08-18, is already a 404; UNVERIFIED whether a
  real page ever existed at that path).
- Mozilla's accessibility content is a support article, "Accessibility features
  in Firefox - Make Firefox and web content work for all users": "Firefox
  includes many features to make the browser and web content accessible to all
  users, including those who have low vision, no vision, or limited ability to
  use a keyboard or mouse." Sections are pure feature how-tos (Using a Keyboard,
  Mouse Shortcuts, Zoom/Fonts and Colors, Themes, …). The live SUMO page serves
  a JS shell to curl; quotes verified via the 2022-11-08 Wayback snapshot [18].
- Notable: a browser vendor with a huge accessibility surface also chose
  features-in-KB over a formal statement.

### The absence survey; the real norm for small/FOSS projects [19]–[22]

Probed live on 2026-11-02, all returning 404 or NXDOMAIN, with Wayback CDX
showing zero captures (i.e. no archived page either):

- Signal: https://signal.org/accessibility/; 404; no captures [19]
- Bitwarden: https://bitwarden.com/accessibility/, /help/accessibility/,
  /help/vpat/; all 404; the bitwarden.com homepage contains **zero** links
  matching "accessib" [20]
- GNOME: https://www.gnome.org/accessibility/; 404;
  https://accessibility.gnome.org/; DNS does not resolve; zero Wayback
  captures (a GNOME statement site is sometimes referenced anecdotally;
  UNVERIFIED, found no trace) [21]
- Thunderbird (thunderbird.net/accessibility/), Mastodon
  (joinmastodon.org/accessibility), Ghost (ghost.org/accessibility/), Element
  (element.io/accessibility), VLC (videolan.org/vlc/accessibility.html),
  LibreOffice (libreoffice.org/accessibility/); all 404 [22]

Notable: among the ~12 small/FOSS projects probed, only Discourse and WordPress
have a page; absence is the genre norm, which makes a good epher page a cheap
differentiator.

## Synthesis: what recurs across genres

Elements and how far they travel:

| Element | Regulated statements (GOV.UK/NHS) | Big-vendor marketing (Apple/MS/Google) | Small/FOSS (Discourse/WordPress/WAI) |
|---|---|---|---|
| Scope sentence ("applies to …") | required | implied (whole company) | sometimes |
| Commitment statement | required | yes, as brand promise | yes, first sentence |
| "What you should be able to do" capability list | signature element (zoom 400%, keyboard, JAWS/NVDA/VoiceOver) | no (features instead) | rare |
| Feature list grouped by need/disability | no | the whole page | no |
| Conformance claim + WCAG version/level | legally fixed wording | off-page (Google's VPATs) | soft ("follow WCAG", "AA where possible") |
| Known limitations + workarounds | required, per-issue with SC + fix date | never | WAI-recommended; WordPress links its tracker instead |
| Feedback channel | required + response time | support desks (chat/phone/ASL) | always; email minimum; Discourse adds "include your browser/AT" |
| AT/browser compatibility & tested environments | inside capability list | implicit | WAI-recommended; in WAI's own statement |
| Assessment method (self-/external) | required (who tested, when) | external audits (unquoted) | WAI: "self-evaluation" |
| Enforcement/complaints route | required (EHRC/EASS) | no | no |
| Statement/review/test dates | required; annual review | no dates | WAI own has none; generator advises ≤ 1 year |
| Technologies relied upon | implicit | no | generator-only; rarely seen in the wild |

Cross-genre invariants, worth treating as the true convention: **(1) a
commitment sentence, (2) a named standard (WCAG 2.2 AA is now the default
everywhere; the generator's default option, the UK requirement, WordPress's
aim), (3) a working feedback channel, and (4) honesty about what doesn't work**
; the last one is what separates the regulated genre from marketing. Two further
recurring mechanics: the statement needs a visible date because freshness is
part of the claim ("should not exceed one year, or it may be considered
unmaintained" [2]; "reviewed at least once a year" [5]), and the page must be
findable under a consistent name from the footer/menu [1]. The EU Web
Accessibility Directive is the common ancestor of the regulated format [4][5];
the European Accessibility Act is now pulling even marketing pages into
mentioning compliance (Microsoft, Google) [10][11].

## Recommended shape for epher.org (observations, not page copy)

What the survey says is realistic for a one-maintainer FOSS calculator site;
observations only, no copy drafted, nothing in `site/` touched:

1. **Genre choice: statement, not marketing.** epher is closest to
   Discourse/WordPress/WAI; the Apple/MS/Google genre exists to sell feature
   breadth epher doesn't have, and the GOV.UK genre's enforcement/disproportionate-burden
   machinery exists because of UK/EU law that doesn't bind a hobby FOSS project.
   The Discourse four-sentence floor plus WAI's five advisable items is the
   sweet spot.
2. **The three required elements are all cheap for epher**: commitment
   (ADR-0009 already declares accessibility an architecture constraint), the
   standard (WCAG 2.2 AA; already the repo's tested target, per
   `docs/accessibility.md`'s per-criterion audit), and contact (GitHub issues +
   maintainer email; Discourse's "include your browser and assistive technology"
   ask is worth copying).
3. **A capability list is the GOV.UK pattern most worth stealing**, because a
   calculator has crisp equivalents: keyboard-first operation, zoom, screen
   reader support, RTL Arabic and the other interface languages; each is a
   user-verifiable claim grounded in the existing audit doc.
4. **A short known-limitations list is the differentiator.** No small project in
   the survey does this well; WAI's five-part limitation record (part / issue /
   why / what we're doing / what to do meanwhile) can be one line each, sourced
   directly from the open items in `docs/accessibility.md`, with tracker links
   the way WordPress does. Honest limitation disclosure is also the
   genre-defining trait of the statements users actually trust.
5. **Conformance wording**: WAI's four-way status (fully/partially/non/not
   assessed) plus "self-evaluation" as the named assessment method (axe +
   VoiceOver hand-testing) matches what epher actually does; no audit firm to
   cite, and none needed at this scale (WAI does the same).
6. **Date it, and plan to re-date it.** One visible statement date; the
   one-year heuristic and GOV.UK's annual-review rule both suggest re-dating
   after major releases is enough maintenance for this size of site.
7. **Keep enforcement, formal approval, organizational-measures lists
   (accessibility officer, training, procurement), VPAT/ATAG documentation, and
   response-time SLAs out**; every one of those exists in the corpus only
   where an organization, a regulator, or a compliance market requires it. A
   modest response-time promise (like WAI's) is optional; a hard SLA is not.
8. **Placement and naming**: WAI's consistency rule supports the planned nav
   label "Accessibility" exactly (not "Accessibility statement"), positioned
   with the other meta pages before Privacy; footer duplication is the other
   standard location. Note the page itself must then meet the bar it declares;
   it will be served in all 8 interface languages like the rest of the nav
   (the WAI Quick Start itself ships in ~20 translations, and GOV.UK's
   multi-domain note shows one statement may cover site + apps).

## Sources

All fetched 2026-11-02 unless noted.

1. W3C WAI, "Developing an Accessibility Statement" (Quick Start):
   https://www.w3.org/WAI/planning/statements/
2. W3C WAI, "Generate an Accessibility Statement" (generator tool):
   https://www.w3.org/WAI/planning/statements/generator/
3. W3C WAI website's own Accessibility Statement:
   https://www.w3.org/WAI/about/accessibility-statement/
4. GOV.UK/GDS, "Sample accessibility statement" (publication page):
   https://www.gov.uk/government/publications/sample-accessibility-statement
5. GOV.UK/GDS, "Sample accessibility statement (for a fictional public sector
   website)" (the sample itself):
   https://www.gov.uk/government/publications/sample-accessibility-statement/sample-accessibility-statement-for-a-fictional-public-sector-website
6. GOV.UK, "Accessibility statement for www.gov.uk":
   https://www.gov.uk/help/accessibility-statement
7. NHS website accessibility statement: https://www.nhs.uk/accessibility/
8. BBC Accessibility Help: https://www.bbc.co.uk/accessibility/
9. Apple Accessibility: https://www.apple.com/accessibility/
10. Microsoft Accessibility: https://www.microsoft.com/accessibility
11. Google Accessibility: https://www.google.com/accessibility/
12. Discourse Accessibility Statement: https://www.discourse.org/accessibility
13. WordPress Accessibility: https://wordpress.org/accessibility/
14. Wikipedia:Manual of Style/Accessibility (Wikipedia:Accessibility redirects
    here): https://en.wikipedia.org/wiki/Wikipedia:Accessibility
15. MediaWiki.org, "Accessibility" (stub):
    https://www.mediawiki.org/wiki/Accessibility
16. Wikimedia Foundation, Policy:Accessibility (empty page):
    https://foundation.wikimedia.org/wiki/Policy:Accessibility
17. mozilla.org /en-US/accessibility (404; sole Wayback capture is a 404,
    2025-08-18): https://www.mozilla.org/en-US/accessibility/ ·
    https://web.archive.org/cdx/search/cdx?url=www.mozilla.org/en-US/accessibility
18. "Accessibility features in Firefox" KB article (live page is a JS shell;
    text via Wayback 2022-11-08):
    https://support.mozilla.org/en-US/kb/accessibility-features-firefox ·
    https://web.archive.org/web/20221108040039/https://support.mozilla.org/en-US/kb/accessibility-features-firefox
19. Signal absence: https://signal.org/accessibility/ (404; zero Wayback
    captures)
20. Bitwarden absence: https://bitwarden.com/accessibility/,
    https://bitwarden.com/help/accessibility/, https://bitwarden.com/help/vpat/
    (all 404); homepage link scan (no "accessib" hrefs): https://bitwarden.com/
21. GNOME absence: https://www.gnome.org/accessibility/ (404);
    https://accessibility.gnome.org/ (NXDOMAIN; zero Wayback captures:
    https://web.archive.org/cdx/search/cdx?url=accessibility.gnome.org/*)
22. Other probed 404s: https://www.thunderbird.net/accessibility/,
    https://joinmastodon.org/accessibility, https://ghost.org/accessibility/,
    https://element.io/accessibility,
    https://www.videolan.org/vlc/accessibility.html,
    https://www.libreoffice.org/accessibility/
23. epher repo context (read for comparison only, not modified):
    docs/adr/0009-accessibility-wcag-22-aa.md, docs/accessibility.md,
    site/index.html (nav order; line 92 "Privacy").
