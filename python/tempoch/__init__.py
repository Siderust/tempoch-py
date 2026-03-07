"""
tempoch: Astronomical Time Primitives for Python

This package provides Python bindings for the tempoch Rust library,
enabling typed astronomical time operations: Julian Date, Modified Julian
Date, UTC conversion, time-scale conversion, and period (interval)
operations — all backed by Rust for performance and correctness.
"""

# Import from the Rust extension module
from tempoch._tempoch import (
    JulianDate,
    ModifiedJulianDate,
    TimePeriod,
    TimeScale,
    NonFiniteTimeError,
    InvalidIntervalError,
    ConversionError,
    convert_timescale,
    tai_minus_utc_py as tai_minus_utc,
    intersect_periods_py as intersect_periods,
    __version__,
)

__all__ = [
    "JulianDate",
    "ModifiedJulianDate",
    "TimePeriod",
    "TimeScale",
    "NonFiniteTimeError",
    "InvalidIntervalError",
    "ConversionError",
    "convert_timescale",
    "tai_minus_utc",
    "intersect_periods",
    "__version__",
]
