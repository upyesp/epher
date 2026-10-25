# Agent notes

Project context lives in `CONTEXT.md`; decisions in `docs/adr/`.

## Codebase graph

Structural questions go through the codebase-memory graph before grep. This repo
indexes to ~5.7k nodes / 17.6k edges across 15 crates, so the graph is much
cheaper than reading `crates/core/src/lib.rs` (the largest single file).

- Project name: `Users-pete-Developer-code-epher`
- Reindex after large refactors: `index_repository(repo_path="<repo root>", mode="moderate")`
- `get_architecture(aspects=["packages"])` for the crate map; `search_graph(label="Function", name_pattern=".*Pattern.*")` to locate symbols; `get_code_snippet` for exact line ranges; `trace_path(direction="inbound")` for impact
- Always constrain degree and dead-code queries with `label="Function"`. Unfiltered, they return folders, YAML and doc files and are unusable
- For fan-in/fan-out rankings use `query_graph` Cypher with `count(r)` and `is_test = false`. `search_graph(min_degree=...)` has been observed to ignore the degree filter and return every row
- Test helpers dominate raw call counts (`eval_str`, `assert_close`, `parse`). Filter `a.is_test = false` or rank by `file_path` to see production structure

Note: the graph's own ADR store (`manage_adr`, currently empty) is separate from
`docs/adr/`, which holds this repo's real decision records. Do not duplicate them.
