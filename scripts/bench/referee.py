#!/usr/bin/env python3
"""50 to 60 digit decimal referee for the calculator benchmark (stage 3).

The parity report's stdlib-only method, reused: decimal at 60 digits, no
third-party dependencies. Computes acceptance values for the W1 and W2
datasets and the W9a prime count, writes scripts/bench/data/acceptance.json.
Gate tolerance (relative 1e-9) is applied by the drivers, not here.
"""
import json
import re
import sys
from decimal import Decimal, getcontext
from pathlib import Path

getcontext().prec = 60

# pi to 70 digits; the dataset arguments are <= 6.1 so reduction is exact here.
PI = Decimal(
    "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089"
)
TWO_PI = 2 * PI


def _reduce(x):
    two_pi = TWO_PI
    n = int(x / two_pi)
    return x - n * two_pi


def d_sin(x):
    x = _reduce(x)
    total = Decimal(0)
    term = x
    k = 1
    while True:
        total += term
        term = -term * x * x / ((2 * k) * (2 * k + 1))
        if abs(term) < Decimal("1e-59"):
            break
        k += 1
        if k > 200:
            raise RuntimeError("sin series did not converge")
    return +total


def d_cos(x):
    x = _reduce(x)
    total = Decimal(1)
    term = Decimal(1)
    k = 1
    while True:
        term = -term * x * x / ((2 * k - 1) * (2 * k))
        total += term
        if abs(term) < Decimal("1e-59"):
            break
        k += 1
        if k > 200:
            raise RuntimeError("cos series did not converge")
    return +total


def d_tan(x):
    return +d_sin(x) / d_cos(x)


def d_ln(x):
    return +x.ln()


def d_exp(x):
    return +x.exp()


def d_sqrt(x):
    return +x.sqrt()


FUNCS = {
    "sin": d_sin,
    "cos": d_cos,
    "tan": d_tan,
    "exp": d_exp,
    "log": d_ln,
    "sqrt": d_sqrt,
}


def eval_line(line):
    """Canonical expression -> Decimal. `^` is power; `log(` is natural log."""
    expr = line.strip().replace("^", "**").replace("log(", "__ln(")
    # Wrap every numeric literal as Decimal so the whole tree evaluates at
    # the referee's precision (python would otherwise use floats).
    expr = re.sub(r"(?<![\w.])(\d+\.\d+)", r'Decimal("\1")', expr)
    return eval(expr, {"__builtins__": {}}, {"Decimal": Decimal, **FUNCS, "__ln": d_ln})  # noqa: S307


def w9a_count():
    flags = bytearray([1]) * 10001
    flags[0] = flags[1] = 0
    p = 2
    while p * p <= 10000:
        if flags[p]:
            for k in range(p * p, 10001, p):
                flags[k] = 0
        p += 1
    return sum(flags)


def main():
    root = Path(__file__).resolve().parent
    data = root / "data"
    out = {"generator_seed": 1, "referee_digits": getcontext().prec}

    w1 = []
    for line in (data / "W1-expressions.txt").read_text().splitlines():
        if line.strip():
            w1.append(f"{eval_line(line):.25e}")
    out["w1"] = w1

    w2 = []
    for line in (data / "W2-transcendental.txt").read_text().splitlines():
        if line.strip():
            w2.append(f"{eval_line(line):.25e}")
    out["w2"] = w2

    out["w9a_count"] = w9a_count()
    (data / "acceptance.json").write_text(json.dumps(out))
    print(f"acceptance.json: w1={len(w1)} w2={len(w2)} w9a_count={out['w9a_count']}")


if __name__ == "__main__":
    sys.exit(main())
