//! Time-scale conversion support.
//!
//! Exposes the `TimeScale` enum and conversion functions to Python,
//! allowing users to convert between JD on different physical time scales
//! (TT, TDB, TAI, TCG, TCB, GPS, UT, etc.).

use pyo3::prelude::*;
use tempoch::qtty::{unit::Second, Second as Seconds};
use tempoch::{
    JulianDate, ModifiedJulianDate, Time, TimeContext, Unix, GPS, JD, MJD, TAI, TCB, TCG, TDB, TT,
    UNIX_EPOCH_JD_DAY, UT1, UTC,
};

use crate::errors::{ensure_finite, map_conversion_error};

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
    /// Legacy name for Julian Date on the TT scale.
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
    /// Universal Time (the UT1 Earth-rotation scale).
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

/// Convert a value between the legacy public representations.
///
/// JD, JDE, MJD, TT, TDB, TAI, TCG, TCB, and UT use day counts. GPS uses
/// seconds since the GPS epoch, while UnixTime uses POSIX seconds. JDE remains
/// as a compatibility alias for a TT Julian Date.
///
/// Args:
///     jd_value: Julian Day number on the source scale.
///     from_scale: source time scale.
///     to_scale: target time scale.
///
/// Returns:
///     float: Julian Day number on the target scale.
#[pyfunction]
pub fn convert_timescale(
    value: f64,
    from_scale: PyTimeScale,
    to_scale: PyTimeScale,
) -> PyResult<f64> {
    ensure_finite(value)?;
    let context = TimeContext::new();
    let tt = to_tt(value, from_scale, &context).map_err(map_conversion_error)?;
    from_tt(tt, to_scale, &context).map_err(map_conversion_error)
}

fn to_tt(
    value: f64,
    representation: PyTimeScale,
    context: &TimeContext,
) -> Result<Time<TT>, tempoch::ConversionError> {
    let tt = match representation {
        PyTimeScale::JD | PyTimeScale::JDE | PyTimeScale::TT => {
            JulianDate::<TT>::new(value).to_j2000s()
        }
        PyTimeScale::MJD => ModifiedJulianDate::<TT>::new(value).to_j2000s(),
        PyTimeScale::TDB => JulianDate::<TDB>::new(value).to::<TT>().to_j2000s(),
        PyTimeScale::TAI => JulianDate::<TAI>::new(value).to::<TT>().to_j2000s(),
        PyTimeScale::TCG => JulianDate::<TCG>::new(value).to::<TT>().to_j2000s(),
        PyTimeScale::TCB => JulianDate::<TCB>::new(value).to::<TT>().to_j2000s(),
        PyTimeScale::GPS => Time::<TAI, GPS>::new(value).to::<TT>().to_j2000s(),
        PyTimeScale::UnixTime => Time::<UTC, Unix>::try_new(Seconds::new(value))?
            .to::<TT>()
            .to_j2000s(),
        PyTimeScale::UT => JulianDate::<UT1>::new(value)
            .to_with::<TT>(context)?
            .to_j2000s(),
    };
    Ok(tt)
}

fn from_tt(
    tt: Time<TT>,
    representation: PyTimeScale,
    context: &TimeContext,
) -> Result<f64, tempoch::ConversionError> {
    let value = match representation {
        PyTimeScale::JD | PyTimeScale::JDE | PyTimeScale::TT => tt.to::<JD>().value(),
        PyTimeScale::MJD => tt.to::<MJD>().value(),
        PyTimeScale::TDB => tt.to::<TDB>().to::<JD>().value(),
        PyTimeScale::TAI => tt.to::<TAI>().to::<JD>().value(),
        PyTimeScale::TCG => tt.to::<TCG>().to::<JD>().value(),
        PyTimeScale::TCB => tt.to::<TCB>().to::<JD>().value(),
        PyTimeScale::GPS => tt.to::<GPS>().raw().value(),
        PyTimeScale::UnixTime => tt.try_to::<Unix>()?.try_raw_with(context)?.value(),
        PyTimeScale::UT => tt.to_with::<UT1>(context)?.to::<JD>().value(),
    };
    Ok(value)
}

/// Get the TAI − UTC leap-second offset for a given Julian Date.
///
/// Args:
///     jd: Julian Date value.
///
/// Returns:
///     float: TAI − UTC in seconds.
#[pyfunction]
pub fn tai_minus_utc_py(jd: f64) -> PyResult<f64> {
    ensure_finite(jd)?;
    let jd_utc = tempoch::qtty::Day::new(jd);
    let unix_seconds = (jd_utc - UNIX_EPOCH_JD_DAY).to::<Second>();
    let utc = Time::<UTC, Unix>::try_new(unix_seconds).map_err(map_conversion_error)?;
    let actual_tai = utc.to::<TAI>().to_j2000s();
    let same_label_tai = JulianDate::<TAI>::new(jd).to_j2000s();
    Ok((actual_tai - same_label_tai).value())
}
