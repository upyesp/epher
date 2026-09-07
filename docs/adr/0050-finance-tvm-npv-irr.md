# ADR-0050: finance — the TVM solver, NPV/IRR, and amortization

- Status: accepted
- Date: 2026-09-02
- Roadmap: feature-gap analysis round 9 (T3.2 finance: TVM any-field
  solver, NPV/IRR, amortization — TI, HP Prime, NumWorks, GeoGebra)

## Context

The last sequenced package is the business-user magnet: the
time-value-of-money solver with any field as the unknown, net present
value and internal rate of return, and amortization. The report calls
it self-contained with no engine work and a moderate i18n burden.

## Decision

### The TVM solver

- The time-value equation uses the TI sign convention (money out is
  negative, money in positive): the balance
  `pv*(1+i)^n + pmt*(1+i*begin)*((1+i)^n - 1)/i + fv` is zero for a
  consistent set. `i` is the per-period rate as a fraction (0.01 is
  1%), and `begin` is the payment timing: 0 = end of period (the
  default), 1 = beginning (annuity due).
- Five functions solve for one field given the other four, each
  taking an optional final `begin` argument:
  `tvm_pmt(n, i, pv, fv)`, `tvm_pv(n, i, pmt, fv)`,
  `tvm_fv(n, i, pv, pmt)`, `tvm_n(i, pv, pmt, fv)`, and
  `tvm_i(n, pv, pmt, fv)`. The linear fields (pv, pmt, fv) have
  closed forms; n and i solve the balance numerically — bisection
  with the factorized balance
  `(pv + pmt*(1+i*begin)/i)*(1+i)^n - pmt*(1+i*begin)/i + fv`
  (which stays finite where the expanded form would overflow), n over
  `[0, 1e7]` and i over `(-0.999999, 1)`; no sign change means a
  domain error naming the searched range.
- The worked example is the classic 8% mortgage: 360 monthly payments
  of 733.76 against a 100,000 loan:
  `tvm_i(360, -100000, 733.76, 0)` answers `0.0066666…` (8%/12).

### NPV, IRR, and amortization

- `npv(rate, flows)` discounts a cash-flow list:
  `sum(flows[k] / (1+rate)^k)` — flow 0 is the present outlay.
- `irr(flows)` finds the rate where npv is zero by bisection over
  `(-0.999999, 1)`; no sign change is a domain error.
- `amort(principal, rate, n, k)` answers the remaining balance after
  k payments of an n-period loan (k = 0 is the principal, k = n is
  zero; the payment itself is `tvm_pmt(principal, rate, n)`).
- `simple_interest(p, r, t)` is `p*r*t` and `compound_interest(p, r,
  n)` is `p*(1+r)^n - p` (NumWorks's two interest applets, one line
  each).
- All ten functions return plain numbers — arithmetic-usable, unlike
  the display strings of linreg and the tests.

## Consequences

- Ten new catalog entries with hints in all eight locales and one
  guide section; no frontend changes (typed input, autocomplete, and
  the keypad stays frozen as ever).
- The rate search caps at 100% per period and the term search at ten
  million periods; out-of-range problems report the searched range.
- Sign conventions follow TI; the guide spells them out with the
  mortgage example so a wrong sign reads as a wrong answer, not a
  mystery.
