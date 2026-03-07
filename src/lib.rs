//! Python bindings for tempoch astronomical time primitives.
//!
//! This crate provides Python bindings for the tempoch Rust library, enabling
//! astronomical time operations in Python with typed time-scale support,
//! Julian Date / Modified Julian Date handling, UTC conversion, and period
//! (interval) operations — all backed by the Rust implementation.

use pyo3::prelude::*;

mod errors;
mod jd;
mod mjd;
mod period;
mod scales;

use jd::PyJulianDate;
use mjd::PyModifiedJulianDate;
use period::{PyTimePeriod, intersect_periods_py};
use scales::{PyTimeScale, convert_timescale, tai_minus_utc_py};

/// tempoch: Astronomical Time Primitives for Python
///
/// This module provides typed astronomical time primitives with typed time
/// scales, Julian Date / Modified Julian Date handling, UTC conversion, and
/// period (interval) operations — all backed by Rust for high performance.
///
/// Example:
/// >>> from tempoch import JulianDate, ModifiedJulianDate
/// >>> j2000 = JulianDate.j2000()
/// >>> mjd = j2000.to_mjd()
/// >>> print(mjd)
/// ModifiedJulianDate(51544.5)
#[pymodule]
fn _tempoch(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Time classes
    m.add_class::<PyJulianDate>()?;
    m.add_class::<PyModifiedJulianDate>()?;
    m.add_class::<PyTimePeriod>()?;
    m.add_class::<PyTimeScale>()?;

    // Exception types
    m.add(
        "NonFiniteTimeError",
        _py.get_type::<errors::NonFiniteTimeError>(),
    )?;
    m.add(
        "InvalidIntervalError",
        _py.get_type::<errors::InvalidIntervalError>(),
    )?;
    m.add(
        "ConversionError",
        _py.get_type::<errors::ConversionError>(),
    )?;

    // Free functions
    m.add_function(wrap_pyfunction!(convert_timescale, m)?)?;
    m.add_function(wrap_pyfunction!(tai_minus_utc_py, m)?)?;
    m.add_function(wrap_pyfunction!(intersect_periods_py, m)?)?;

    // Version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
