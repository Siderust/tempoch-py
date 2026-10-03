//! Python wrappers for `Period<MJD>` (time intervals on MJD scale).

use pyo3::prelude::*;
use tempoch::qtty::unit::{Day, Hour, Second};
use tempoch::{Interval, ModifiedJulianDate, Period, MJD, TT, UTC};

use crate::errors::{ensure_finite, map_conversion_error, map_invalid_interval_error};
use crate::jd::PyJulianDate;
use crate::mjd::PyModifiedJulianDate;

/// A time period defined by start and end Modified Julian Dates.
///
/// Periods are half-open intervals [start, end) that support duration
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
    inner: Period<TT>,
}

impl PyTimePeriod {
    /// Create from an inner Rust `Period<MJD>`.
    pub fn from_inner(inner: Period<TT>) -> Self {
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
        ensure_finite(start_mjd)?;
        ensure_finite(end_mjd)?;
        let start = ModifiedJulianDate::<TT>::new(start_mjd).to_j2000s();
        let end = ModifiedJulianDate::<TT>::new(end_mjd).to_j2000s();
        let inner = Interval::try_new(start, end).map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// Create a time period from two `ModifiedJulianDate` objects.
    #[staticmethod]
    fn from_mjd(start: &PyModifiedJulianDate, end: &PyModifiedJulianDate) -> PyResult<Self> {
        let inner = Interval::try_new(start.inner.to_j2000s(), end.inner.to_j2000s())
            .map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// Create a time period from two `JulianDate` objects.
    ///
    /// The JD values are converted to MJD internally.
    #[staticmethod]
    fn from_jd(start: &PyJulianDate, end: &PyJulianDate) -> PyResult<Self> {
        let start_mjd = start.inner.to_j2000s();
        let end_mjd = end.inner.to_j2000s();
        let inner = Interval::try_new(start_mjd, end_mjd).map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// Create a time period from two UTC datetime strings.
    #[staticmethod]
    fn from_utc(start_utc: &str, end_utc: &str) -> PyResult<Self> {
        use chrono::{DateTime, FixedOffset, Utc};
        let start_dt: DateTime<FixedOffset> = start_utc.parse().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid start UTC: {e}"))
        })?;
        let end_dt: DateTime<FixedOffset> = end_utc.parse().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid end UTC: {e}"))
        })?;
        let start = tempoch::Time::<UTC>::try_from_chrono(start_dt.with_timezone(&Utc))
            .map_err(map_conversion_error)?
            .to::<TT>();
        let end = tempoch::Time::<UTC>::try_from_chrono(end_dt.with_timezone(&Utc))
            .map_err(map_conversion_error)?
            .to::<TT>();
        let inner = Interval::try_new(start, end).map_err(map_invalid_interval_error)?;
        Ok(Self { inner })
    }

    /// The start of the period as a `ModifiedJulianDate`.
    #[getter]
    fn start(&self) -> PyModifiedJulianDate {
        PyModifiedJulianDate::from_inner(self.inner.start.to::<MJD>())
    }

    /// The end of the period as a `ModifiedJulianDate`.
    #[getter]
    fn end(&self) -> PyModifiedJulianDate {
        PyModifiedJulianDate::from_inner(self.inner.end.to::<MJD>())
    }

    /// The start MJD value (float).
    #[getter]
    fn start_mjd(&self) -> f64 {
        self.inner.start.to::<MJD>().value()
    }

    /// The end MJD value (float).
    #[getter]
    fn end_mjd(&self) -> f64 {
        self.inner.end.to::<MJD>().value()
    }

    /// Duration of the period in days.
    fn duration_days(&self) -> f64 {
        (self.inner.end - self.inner.start).to::<Day>().value()
    }

    /// Duration of the period in seconds.
    fn duration_seconds(&self) -> f64 {
        (self.inner.end - self.inner.start).to::<Second>().value()
    }

    /// Duration of the period in hours.
    fn duration_hours(&self) -> f64 {
        (self.inner.end - self.inner.start).to::<Hour>().value()
    }

    /// Convert start/end to UTC datetime strings.
    ///
    /// Returns:
    ///     tuple[str, str]: (start_utc, end_utc) in ISO 8601 format.
    fn to_utc(&self) -> PyResult<(String, String)> {
        let start_utc = self
            .inner
            .start
            .to::<UTC>()
            .try_to_chrono()
            .map_err(map_conversion_error)?;
        let end_utc = self
            .inner
            .end
            .to::<UTC>()
            .try_to_chrono()
            .map_err(map_conversion_error)?;
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
        if !mjd.is_finite() {
            return false;
        }
        let t = ModifiedJulianDate::<TT>::new(mjd).to_j2000s();
        t >= self.inner.start && t < self.inner.end
    }

    /// Check if this period contains a `ModifiedJulianDate`.
    fn contains_mjd(&self, mjd: &PyModifiedJulianDate) -> bool {
        let instant = mjd.inner.to_j2000s();
        instant >= self.inner.start && instant < self.inner.end
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
            self.inner.start.to::<MJD>().value(),
            self.inner.end.to::<MJD>().value()
        )
    }

    fn __str__(&self) -> String {
        format!(
            "TimePeriod(MJD {} to {})",
            self.inner.start.to::<MJD>().value(),
            self.inner.end.to::<MJD>().value()
        )
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((
            cls,
            (
                self.inner.start.to::<MJD>().value(),
                self.inner.end.to::<MJD>().value(),
            ),
        ))
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner
            .start
            .to::<MJD>()
            .value()
            .to_bits()
            .hash(&mut hasher);
        self.inner
            .end
            .to::<MJD>()
            .value()
            .to_bits()
            .hash(&mut hasher);
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
    let rust_periods: Vec<Period<TT>> = periods.iter().map(|p| p.inner).collect();
    let bound_period = bounds.inner;
    rust_periods
        .iter()
        .filter_map(|p| p.intersection(&bound_period))
        .map(PyTimePeriod::from_inner)
        .collect()
}
