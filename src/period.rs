//! Python wrappers for `Period<MJD>` (time intervals on MJD scale).

use pyo3::prelude::*;
use tempoch::{Interval, Period, Time, MJD};

use crate::errors::{map_invalid_interval_error, utc_conversion_failed};
use crate::jd::PyJulianDate;
use crate::mjd::PyModifiedJulianDate;

/// A time period defined by start and end Modified Julian Dates.
///
/// Periods are closed intervals [start, end] that support duration
/// calculation, intersection, and scale conversion.
///
/// Examples:
///     >>> from tempoch import TimePeriod
///     >>> p = TimePeriod(59000.0, 59001.0)
///     >>> p.duration_days()
///     1.0
///     >>> p.start
///     ModifiedJulianDate(59000.0)
#[pyclass(name = "TimePeriod", module = "tempoch", from_py_object)]
#[derive(Clone, Copy)]
pub struct PyTimePeriod {
    inner: Period<MJD>,
}

impl PyTimePeriod {
    /// Create from an inner Rust `Period<MJD>`.
    pub fn from_inner(inner: Period<MJD>) -> Self {
        Self { inner }
    }
}

#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyTimePeriod {
    /// Create a new time period from start and end MJD values.
    ///
    /// Args:
    ///     start_mjd: start time as Modified Julian Date.
    ///     end_mjd: end time as Modified Julian Date.
    ///
    /// Raises:
    ///     InvalidIntervalError: if start > end.
    #[new]
    fn new(start_mjd: f64, end_mjd: f64) -> PyResult<Self> {
        let start = Time::<MJD>::new(start_mjd);
        let end = Time::<MJD>::new(end_mjd);
        let inner = Interval::try_new(start, end).map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// Create a time period from two `ModifiedJulianDate` objects.
    #[staticmethod]
    fn from_mjd(start: &PyModifiedJulianDate, end: &PyModifiedJulianDate) -> PyResult<Self> {
        let inner =
            Interval::try_new(start.inner, end.inner).map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// Create a time period from two `JulianDate` objects.
    ///
    /// The JD values are converted to MJD internally.
    #[staticmethod]
    fn from_jd(start: &PyJulianDate, end: &PyJulianDate) -> PyResult<Self> {
        let start_mjd = start.inner.to::<MJD>();
        let end_mjd = end.inner.to::<MJD>();
        let inner = Interval::try_new(start_mjd, end_mjd).map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// Create a time period from two UTC datetime strings.
    #[staticmethod]
    fn from_utc(start_utc: &str, end_utc: &str) -> PyResult<Self> {
        use chrono::{DateTime, Utc};
        let start_dt: DateTime<Utc> = start_utc.parse::<DateTime<Utc>>().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid start UTC: {e}"))
        })?;
        let end_dt: DateTime<Utc> = end_utc.parse::<DateTime<Utc>>().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid end UTC: {e}"))
        })?;
        let start = Time::<MJD>::from_utc(start_dt);
        let end = Time::<MJD>::from_utc(end_dt);
        let inner = Interval::try_new(start, end).map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// The start of the period as a `ModifiedJulianDate`.
    #[getter]
    fn start(&self) -> PyModifiedJulianDate {
        PyModifiedJulianDate::from_inner(self.inner.start)
    }

    /// The end of the period as a `ModifiedJulianDate`.
    #[getter]
    fn end(&self) -> PyModifiedJulianDate {
        PyModifiedJulianDate::from_inner(self.inner.end)
    }

    /// The start MJD value (float).
    #[getter]
    fn start_mjd(&self) -> f64 {
        self.inner.start.value()
    }

    /// The end MJD value (float).
    #[getter]
    fn end_mjd(&self) -> f64 {
        self.inner.end.value()
    }

    /// Duration of the period in days.
    fn duration_days(&self) -> f64 {
        self.inner.duration().value()
    }

    /// Duration of the period in seconds.
    fn duration_seconds(&self) -> f64 {
        use qtty::Second;
        self.inner.duration().to::<Second>().value()
    }

    /// Duration of the period in hours.
    fn duration_hours(&self) -> f64 {
        use qtty::Hour;
        self.inner.duration().to::<Hour>().value()
    }

    /// Convert start/end to UTC datetime strings.
    ///
    /// Returns:
    ///     tuple[str, str]: (start_utc, end_utc) in ISO 8601 format.
    fn to_utc(&self) -> PyResult<(String, String)> {
        let start_utc = self
            .inner
            .start
            .to_utc()
            .ok_or_else(utc_conversion_failed)?;
        let end_utc = self.inner.end.to_utc().ok_or_else(utc_conversion_failed)?;
        Ok((start_utc.to_rfc3339(), end_utc.to_rfc3339()))
    }

    /// Compute the intersection with another period.
    ///
    /// Returns:
    ///     TimePeriod or None: the intersection, or None if the periods don't overlap.
    fn intersection(&self, other: &PyTimePeriod) -> Option<Self> {
        self.inner
            .intersection(&other.inner)
            .map(|p| Self { inner: p })
    }

    /// Check if this period contains a given MJD instant.
    fn contains(&self, mjd: f64) -> bool {
        let t = Time::<MJD>::new(mjd);
        t >= self.inner.start && t <= self.inner.end
    }

    /// Check if this period contains a `ModifiedJulianDate`.
    fn contains_mjd(&self, mjd: &PyModifiedJulianDate) -> bool {
        mjd.inner >= self.inner.start && mjd.inner <= self.inner.end
    }

    fn __eq__(&self, other: &PyTimePeriod) -> bool {
        self.inner == other.inner
    }

    fn __ne__(&self, other: &PyTimePeriod) -> bool {
        self.inner != other.inner
    }

    fn __repr__(&self) -> String {
        format!(
            "TimePeriod({}, {})",
            self.inner.start.value(),
            self.inner.end.value()
        )
    }

    fn __str__(&self) -> String {
        format!(
            "TimePeriod(MJD {} to {})",
            self.inner.start.value(),
            self.inner.end.value()
        )
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((cls, (self.inner.start.value(), self.inner.end.value())))
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner.start.value().to_bits().hash(&mut hasher);
        self.inner.end.value().to_bits().hash(&mut hasher);
        hasher.finish()
    }
}

/// Compute the intersection of a list of periods.
///
/// Args:
///     periods: list of `TimePeriod` objects. Must be sorted by start time
///              and non-overlapping.
///     bounds: the bounding `TimePeriod` to intersect against.
///
/// Returns:
///     list[TimePeriod]: intersected periods within the bounds.
#[pyfunction]
pub fn intersect_periods_py(
    periods: Vec<PyTimePeriod>,
    bounds: &PyTimePeriod,
) -> Vec<PyTimePeriod> {
    let rust_periods: Vec<Period<MJD>> = periods.iter().map(|p| p.inner).collect();
    let bound_period = bounds.inner;
    rust_periods
        .iter()
        .filter_map(|p| p.intersection(&bound_period))
        .map(PyTimePeriod::from_inner)
        .collect()
}
