//! Time-scale conversion support.
//!
//! Exposes the `TimeScale` enum and conversion functions to Python,
//! allowing users to convert between JD on different physical time scales
//! (TT, TDB, TAI, TCG, TCB, GPS, UT, etc.).

use pyo3::prelude::*;
use tempoch::{Time, UnixTime, GPS, JD, JDE, MJD, TAI, TCB, TCG, TDB, TT, UT};

/// Enumeration of supported astronomical time scales.
///
/// Used with `JulianDate.to_scale()` and `JulianDate.from_scale()` to convert
/// between different time representations.
#[pyclass(
    name = "TimeScale",
    module = "tempoch",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PyTimeScale {
    /// Julian Date (identity scale).
    JD = 0,
    /// Julian Ephemeris Day.
    JDE = 1,
    /// Modified Julian Date (JD − 2,400,000.5).
    MJD = 2,
    /// Barycentric Dynamical Time.
    TDB = 3,
    /// Terrestrial Time.
    TT = 4,
    /// International Atomic Time.
    TAI = 5,
    /// Geocentric Coordinate Time (IAU 2000).
    TCG = 6,
    /// Barycentric Coordinate Time (IAU 2006).
    TCB = 7,
    /// GPS Time.
    GPS = 8,
    /// Unix/POSIX Time (seconds since 1970-01-01).
    UnixTime = 9,
    /// Universal Time (Earth rotation, applies ΔT).
    UT = 10,
}

#[pymethods]
impl PyTimeScale {
    fn __repr__(&self) -> String {
        format!("TimeScale.{:?}", self)
    }

    fn __str__(&self) -> &'static str {
        match self {
            Self::JD => "JD",
            Self::JDE => "JDE",
            Self::MJD => "MJD",
            Self::TDB => "TDB",
            Self::TT => "TT",
            Self::TAI => "TAI",
            Self::TCG => "TCG",
            Self::TCB => "TCB",
            Self::GPS => "GPS",
            Self::UnixTime => "UnixTime",
            Self::UT => "UT",
        }
    }
}

/// Convert a JD value from one time scale to another.
///
/// The value is interpreted as a Julian Day number on the source scale,
/// and returned as a Julian Day number on the target scale.
///
/// Args:
///     jd_value: Julian Day number on the source scale.
///     from_scale: source time scale.
///     to_scale: target time scale.
///
/// Returns:
///     float: Julian Day number on the target scale.
#[pyfunction]
pub fn convert_timescale(jd_value: f64, from_scale: PyTimeScale, to_scale: PyTimeScale) -> f64 {
    // Route: source → JD(TT) → target
    // First convert to JD(TT), then from JD(TT) to target scale
    let jd_tt = match from_scale {
        PyTimeScale::JD => Time::<JD>::new(jd_value).value(),
        PyTimeScale::JDE => Time::<JDE>::new(jd_value).to::<JD>().value(),
        PyTimeScale::MJD => Time::<MJD>::new(jd_value).to::<JD>().value(),
        PyTimeScale::TDB => Time::<TDB>::new(jd_value).to::<JD>().value(),
        PyTimeScale::TT => Time::<TT>::new(jd_value).to::<JD>().value(),
        PyTimeScale::TAI => Time::<TAI>::new(jd_value).to::<JD>().value(),
        PyTimeScale::TCG => Time::<TCG>::new(jd_value).to::<JD>().value(),
        PyTimeScale::TCB => Time::<TCB>::new(jd_value).to::<JD>().value(),
        PyTimeScale::GPS => Time::<GPS>::new(jd_value).to::<JD>().value(),
        PyTimeScale::UnixTime => Time::<UnixTime>::new(jd_value).to::<JD>().value(),
        PyTimeScale::UT => Time::<UT>::new(jd_value).to::<JD>().value(),
    };

    match to_scale {
        PyTimeScale::JD => jd_tt,
        PyTimeScale::JDE => Time::<JD>::new(jd_tt).to::<JDE>().value(),
        PyTimeScale::MJD => Time::<JD>::new(jd_tt).to::<MJD>().value(),
        PyTimeScale::TDB => Time::<JD>::new(jd_tt).to::<TDB>().value(),
        PyTimeScale::TT => Time::<JD>::new(jd_tt).to::<TT>().value(),
        PyTimeScale::TAI => Time::<JD>::new(jd_tt).to::<TAI>().value(),
        PyTimeScale::TCG => Time::<JD>::new(jd_tt).to::<TCG>().value(),
        PyTimeScale::TCB => Time::<JD>::new(jd_tt).to::<TCB>().value(),
        PyTimeScale::GPS => Time::<JD>::new(jd_tt).to::<GPS>().value(),
        PyTimeScale::UnixTime => Time::<JD>::new(jd_tt).to::<UnixTime>().value(),
        PyTimeScale::UT => Time::<JD>::new(jd_tt).to::<UT>().value(),
    }
}

/// Get the TAI − UTC leap-second offset for a given Julian Date.
///
/// Args:
///     jd: Julian Date value.
///
/// Returns:
///     float: TAI − UTC in seconds.
#[pyfunction]
pub fn tai_minus_utc_py(jd: f64) -> f64 {
    tempoch::tai_minus_utc(jd)
}
