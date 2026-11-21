# Math and code-block support for markdown in VS Code extensions

Researched 2026-11-21. Question: what do existing VS Code extensions provide
around code blocks (especially math) in markdown files, how does epher's
markdown support compare, and are the fence identifiers `math` and `maths`
already claimed?

Method: primary sources only. VS Code's bundled extension source
(microsoft/vscode), each extension's repo README and package.json fetched
from GitHub, install counts from the Visual Studio Marketplace gallery API
(extensionquery, 2026-11-21). Claims were checked in the fetched source, not
in secondary write-ups.

## The field

| Extension | Installs | What it adds to markdown |
|---|---:|---|
| VS Code built-in "Markdown Math" (`markdown-math`, bundled) | n/a | KaTeX rendering of `$...$`, `$$...$$`, and ` ```math ` fences in the built-in preview and notebook cells; `markdown.math.enabled` (default true), `markdown.math.macros`; ships TextMate grammars so math fences get TeX coloring |
| yzhang.markdown-all-in-one | 14,990,195 | Markdown authoring (TOC, lists, tables). Math: editor-side KaTeX function completions plus `katex.macros`; README defers rendering to Markdown+Math |
| shd101wyy.markdown-preview-enhanced | 10,692,923 | Math typesetting (KaTeX/MathJax), mermaid/PlantUML, pandoc, exports, presentations, and **code chunks**: ` ```{python} ` style fences run with shift-enter (run all: ctrl-shift-enter), output embedded in the preview below the chunk |
| bierner.markdown-mermaid | 5,530,687 | Renders ` ```mermaid ` fences in the markdown preview (preview scripts + markdown-it plugin + notebook renderer). Contributes no grammars or languages; coloring comes from a separate extension |
| ms-dotnettools.dotnet-interactive-vscode (Polyglot Notebooks) | 1,984,111 | Executable polyglot cells, but in notebook files, not markdown fences. No documented markdown-fence execution in the README |
| bpruitt-goddard.mermaid-markdown-syntax-highlighting | 846,995 | TextMate **injection grammar** into markdown (`injectTo: ['text.html.markdown']`, scope `markdown.mermaid.codeblock`) so mermaid fences get mermaid coloring; same mechanism epher's vscode client uses |
| goessner.mdmath (Markdown+Math) | 431,189 | TeX math typeset in the built-in viewer via markdown-it-texmath + KaTeX (MathJax optional); inline and display delimiters user-configurable; scroll sync. Rendering only |
| koehlma.markdown-math | 114,382 | KaTeX in the built-in preview, `$...$` and `$$...$$` delimiters only |
| stateful.runme | 45,552 | Opens markdown files as runnable notebooks: annotated code blocks (shell etc.) get a play button and execute, outputs land in the notebook UX |
| NikolaiFedorov.markdown-kotlin | 129 | Kotlin coloring inside ` ```kotlin `/` ```kt ` fences in markdown (injection), zero config |

## Comparison with epher

| Capability | Math extensions (built-in, mdmath, koehlma, MPE) | Fence-highlighting extensions (mermaid, markdown-kotlin) | Execution extensions (MPE chunks, runme, Polyglot) | epher (ADR-0088) |
|---|---|---|---|---|
| Fence syntax coloring | math fences only (built-in ships md-math-fence grammar) | yes, via TextMate injection into markdown | varies (MPE: no; runme: no; Polyglot: n/a) | yes, injection grammar with `L:text.html.markdown` selector |
| Renders in markdown preview | yes (KaTeX/MathJax/mermaid) | mermaid: yes | MPE: yes (outputs in preview) | no; results are LSP-side, not preview |
| Executes code blocks | no | no | yes, chunk- or cell-scoped | yes: all fences in a document form one virtual script, projected server-side |
| Inline results in the editor (no preview) | no | no | no | yes: answers as inlay hints per statement |
| Run surface (lens/commands) | no | no | MPE/runme: play/run commands | yes: CodeLens, Ctrl+Enter, results pane |
| Debug | no | no | runme: runbooks | yes: F5 to the debug console |
| Hover, completion, diagnostics inside fences | no | no | partial (Polyglot has LSP hovers, in notebooks) | yes: full LSP surface with markdown line mapping |
| Interpreter | none (typesetting only) | none | python/shell/.NET per chunk/cell | the epher engine, one script per document |

The honest summary: math extensions decorate; mermaid/kotlin extensions
color; chunk extensions run isolated snippets. Epher is alone in treating a
markdown document's fences as a single script with a complete LSP surface
(inline answers, diagnostics mapped to markdown lines, hover, completion,
run, debug). Nothing researched does all of that.

## Are `math` and `maths` taken as fence identifiers?

`math` is **taken** in VS Code. The bundled markdown-math extension passes
`enableFencedBlocks: true` to `@vscode/markdown-it-katex` (extensions/
markdown-math/src/extension.ts, microsoft/vscode main), so the built-in
preview renders ` ```math ` fences as typeset math with `markdown.math.enabled`
defaulting to true, and the same fences get TeX coloring from the bundled
`md-math-fence.tmLanguage.json`. A future epher ` ```math ` fence would share
the identifier: the preview would typeset it as math while epher's LSP
evaluated it. Coexistence is technically possible (the two surfaces do not
fight over the editor text), but users would see typeset math and inline
answers at once, and expectations set by the built-in (TeX syntax) would not
match an epher fence. Notably GitHub does NOT render ` ```math ` (its docs
support only `$...$` and `$$...$$`), so ` ```math ` blocks in repo READMEs
show as plain code there.

`maths` is **not taken** by anything authoritative. The vscode repo has no
`maths` language or fence handling; none of the researched math extensions
claim it; GitHub-wide code search finds 87 informal ` ```maths ` usages in
Markdown documents (cheat sheets, course notes) but no extension or renderer
claiming the identifier. Informal usage means some readers will already
associate it with math, but nothing enforces it.

Epher-side, both are free today: the fence scanner matches only when the
info string's first word is `epher` (crates/lsp/src/markdown.rs, tested by
`other_languages_fences_are_ignored`), so ` ```math `/` ```maths ` fences in
markdown are inert. Adopting either later means extending the scanner's
language match (the single source of truth all client gates mirror per
ADR-0088) and deciding server semantics, not touching client filter
plumbing.

## Recommendation sketch (if epher ever wants a math fence)

`maths` avoids colliding with the built-in's typeset-in-preview behavior;
`math` would put epher fences inside the one identifier VS Code itself
renders, which is either a feature (coloring and preview come free) or a
conflict (results plus typeset math on the same fence, TeX-shaped
expectations). Either way the change is confined to the fence scanner and
the engine; client gates follow the mirrored rule. For search engines and
docs discoverability, `math` is the term users type; `maths` is the term
that keeps the two worlds separate.

## Sources

- microsoft/vscode, extensions/markdown-math/{package.json, package.nls.json, src/extension.ts, syntaxes/}, main branch, fetched 2026-11-21. `enableFencedBlocks: true`; `markdown.math.enabled` default true; language `markdown-math`; md-math-fence grammar.
- VS Code docs, "Markdown" page (code.visualstudio.com/docs/languages/markdown), fetched 2026-11-21: "VS Code's built-in Markdown preview renders math equations using KaTeX."
- github/docs, writing-mathematical-expressions.md, main branch, fetched 2026-11-21: inline `$...$` and `` $`...`$ ``, block `$$...$$`; no math fence.
- goessner/mdmath readme (master), fetched 2026-11-21.
- koehlma/vscode-markdown-math readme, fetched 2026-11-21.
- yzhang-gh/vscode-markdown readme (master), fetched 2026-11-21.
- shd101wyy/vscode-markdown-preview-enhanced readme (master), fetched 2026-11-21: code chunk keybindings, math typesetting.
- mjbvz/vscode-markdown-mermaid package.json (master), fetched 2026-11-21: contributes previewScripts/markdownItPlugins/notebookRenderer only.
- bpruitt-goddard/vscode-mermaid-syntax-highlight package.json (master), fetched 2026-11-21: `injectTo: ['text.html.markdown']`.
- dotnet/interactive readme (main), fetched 2026-11-21: no markdown-fence execution documented.
- runmedev/vscode-runme readme (main), fetched 2026-11-21: markdown files as runnable notebooks.
- Unril/markdown-kotlin readme, fetched 2026-11-21: kotlin fence highlighting via injection.
- Visual Studio Marketplace gallery API (extensionquery), 2026-11-21: install counts and ratings above.
- GitHub code search for ` ```maths ` in Markdown files, 2026-11-21: 87 informal hits, no claiming extension.
