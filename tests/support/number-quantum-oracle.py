#!/usr/bin/env python3
"""Verify fixed-precision, half-up quantum expectations using exact rational arithmetic."""

from fractions import Fraction
import json
import sys


if hasattr(sys, "set_int_max_str_digits"):
    sys.set_int_max_str_digits(0)

corpus = json.load(sys.stdin)
checked = 0
for case_index, case in enumerate(corpus["cases"]):
    quantum = int(case["quantum"])
    scale = case["fraction_digits"]
    for sample_index, sample in enumerate(case["samples"]):
        scaled = abs(Fraction(sample["input"])) * 10**scale
        numerator, denominator = scaled.numerator, scaled.denominator
        units = (2 * numerator + denominator * quantum) // (2 * denominator * quantum)
        digits = str(units * quantum).rjust(scale + 1, "0")
        expected = digits if scale == 0 else digits[:-scale] + "." + digits[-scale:]
        if sample["input"].startswith("-"):
            expected = "-" + expected
        assert expected == sample["expected"], (case_index, sample_index)
        checked += 1
print(f"Verified {checked} exact quantum-rounding expectations.")
