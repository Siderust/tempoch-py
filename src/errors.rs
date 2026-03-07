//! Error handling utilities for tempoch Python bindings.
//!
//! Maps Rust error types to Python exceptions, keeping the FFI boundary clean.

use pyo3::exceptions::*;
use pyo3::prelude::*;

// ── Custom exception types ────────────────────────────────────────────────

pyo3::create_exception!(
    tempoch,
    NonFiniteTimeError,
    PyValueError,
    "Time value must be finite (not NaN or infinity)."
);
pyo3::create_exception!(
    tempoch,
    InvalidIntervalError,
    PyValueError,
    "Interval start must not be after end."
);
pyo3::create_exception!(
    tempoch,
    ConversionError,
    PyValueError,
    "Time conversion out of representable range."
);

// ── Mapping helpers ───────────────────────────────────────────────────────

/// Map a Rust `NonFiniteTimeError` to a Python `NonFiniteTimeError`.
pub fn map_non_finite_error(e: tempoch::NonFiniteTimeError) -> PyErr {
    NonFiniteTimeError::new_err(e.to_string())
}

/// Map a Rust `InvalidIntervalError` to a Python `InvalidIntervalError`.
pub fn map_invalid_interval_error(e: tempoch::InvalidIntervalError) -> PyErr {
    InvalidIntervalError::new_err(e.to_string())
}

/// Map a Rust `ConversionError` to a Python `ConversionError`.
#[allow(dead_code)]
pub fn map_conversion_error(e: tempoch::ConversionError) -> PyErr {
    ConversionError::new_err(e.to_string())
}

/// Create an error for UTC conversion failure.
pub fn utc_conversion_failed() -> PyErr {
    ConversionError::new_err("UTC conversion failed: time value out of representable range")
}
