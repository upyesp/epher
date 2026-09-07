# ADR-0049: matrices

- Status: accepted
- Date: 2026-09-02
- Roadmap: feature-gap analysis round 8 (T2.3 matrices — NumWorks's
  minimal set is the floor; eigenvalues deferred per the report)

## Context

Eight of the nine researched apps have matrices, and the expectation
shape is NumWorks's minimal set: literal entry, inverse, determinant,
transpose, trace, dim, and (r)ref, with linear systems solved through
rref. The report's scope: a matrix Value, arithmetic, and the six
functions, with eigenvalues later.

## Decision

### The value and the literal

- New `Value::Matrix { rows: usize, cols: usize, data: Vec<f64> }`
  (row-major, floats only like lists — ADR-0044's column rule).
- The literal is the row-of-rows spelling the web calculators use:
  `[[1, 2], [3, 4]]` — an expression-start `[` begins a matrix whose
  rows are bracket lists; every row must have the same length (a type
  error otherwise, like list shape). `M[i]` indexes a row as a list
  (1-based, so `M[2][1]` is the element at row 2, column 1 — the
  existing postfix index composes), and `dim(M)` answers `{rows,
  cols}` as a list.

### Arithmetic

- `M + N` and `M - N` are elementwise with matching shapes (else a
  type error); `M * N` is the matrix product (m×n times n×p, else a
  type error); `M * c` and `c * M` scale elementwise, `M / c` too,
  and dividing by a matrix is an error. Unary minus negates
  elementwise. `M ^ n` is the matrix power for a whole `n >= 0`
  (n = 0 gives the identity, so the power needs square matrices);
  anything else is an error. Ordering comparisons reject matrices;
  `==` and `!=` compare whole matrices.

### The functions

- `det(M)` — LU with partial pivoting; square only.
- `inv(M)` — Gauss-Jordan on the augmented identity; singular
  matrices are a domain error.
- `transpose(M)`, `trace(M)` (square), `dim(M)` (the `{rows, cols}`
  list).
- `rref(M)` — reduced row echelon form; `ref(M)` — row echelon
  (forward elimination). Linear systems solve through rref on the
  augmented matrix, the TI/NumWorks pattern:
  `rref([[2, 1, 5], [1, -1, 1]])` reads `x = 2, y = 1` off
  `[[1, 0, 2], [0, 1, 1]]`.

## Consequences

- Matrices are floats-only, exact fractions display inside them like
  lists (`inv` of an integer matrix shows `1/2` when the toggle is
  on), and the keypad stays frozen — matrices are typed and
  documented in the guide.
- The sampler, calculus, and table paths already drop non-float
  values, so a matrix in a graph command is an empty plot rather than
  a crash.
- Seven new catalog entries with hints in all eight locales; one new
  guide section.
