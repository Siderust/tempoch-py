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

/// Reject a non-finite scalar before passing it to tempoch.
pub fn ensure_finite(value: f64) -> PyResult<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(NonFiniteTimeError::new_err("time value must be finite"))
    }
}

/// Map a Rust `InvalidIntervalError` to a Python `InvalidIntervalError`.
pub fn map_invalid_interval_error(e: tempoch::InvalidIntervalError) -> PyErr {
    InvalidIntervalError::new_err(e.to_string())
}

/// Map a Rust `ConversionError` to a Python `ConversionError`.
pub fn map_conversion_error(e: tempoch::ConversionError) -> PyErr {
    ConversionError::new_err(e.to_string())
}
