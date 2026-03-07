//! Python wrappers for `Time<JD>` (Julian Date).

use pyo3::prelude::*;
use pyo3::Py;
use tempoch::{JulianDate, Time, JD, MJD};

use crate::errors::{map_non_finite_error, utc_conversion_failed};
use crate::mjd::PyModifiedJulianDate;

/// A Julian Date — continuous day count since the Julian Period epoch.
///
/// Wraps the Rust `Time<JD>` type for Python with full arithmetic,
/// comparison, UTC conversion, and scale-conversion support.
///
/// Examples:
///     >>> from tempoch import JulianDate
///     >>> j2000 = JulianDate.j2000()
///     >>> print(j2000)
///     JulianDate(2451545.0)
///     >>> utc = j2000.to_utc()
///     >>> print(utc)
///     2000-01-01 11:58:55.816000+00:00
#[pyclass(name = "JulianDate", module = "tempoch", from_py_object)]
#[derive(Clone, Copy)]
pub struct PyJulianDate {
    pub(crate) inner: JulianDate,
}

impl PyJulianDate {
    /// Create from an inner Rust `JulianDate`.
    pub fn from_inner(inner: JulianDate) -> Self {
        Self { inner }
    }
}

#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyJulianDate {
    /// Create a new Julian Date from a day number.
    ///
    /// Args:
    ///     value: Julian Day number (days since the Julian Period epoch).
    ///
    /// Raises:
    ///     NonFiniteTimeError: if the value is NaN or infinite.
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        let inner = Time::<JD>::try_new(value).map_err(map_non_finite_error)?;
        Ok(Self { inner })
    }

    /// The J2000.0 epoch: JD 2451545.0 (2000-01-01T12:00:00 TT).
    #[staticmethod]
    fn j2000() -> Self {
        Self {
            inner: JulianDate::J2000,
        }
    }

    /// The raw day-number value.
    #[getter]
    fn value(&self) -> f64 {
        self.inner.value()
    }

    /// Convert to Modified Julian Date.
    fn to_mjd(&self) -> PyModifiedJulianDate {
        PyModifiedJulianDate::from_inner(self.inner.to::<MJD>())
    }

    /// Convert to a UTC datetime string (ISO 8601).
    ///
    /// Returns:
    ///     str: UTC datetime in ISO 8601 format.
    ///
    /// Raises:
    ///     ConversionError: if the value is outside chrono's representable range.
    fn to_utc(&self) -> PyResult<String> {
        let dt = self.inner.to_utc().ok_or_else(utc_conversion_failed)?;
        Ok(dt.to_rfc3339())
    }

    /// Convert to a Python `datetime.datetime` object (UTC).
    ///
    /// Returns:
    ///     datetime.datetime: UTC datetime with timezone info.
    ///
    /// Raises:
    ///     ConversionError: if the value is outside the representable range.
    fn to_datetime<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dt = self.inner.to_utc().ok_or_else(utc_conversion_failed)?;
        let datetime_mod = py.import("datetime")?;
        let datetime_cls = datetime_mod.getattr("datetime")?;
        let tz = datetime_mod.getattr("timezone")?.getattr("utc")?;
        datetime_cls.call_method1(
            "fromtimestamp",
            (
                dt.timestamp() as f64 + dt.timestamp_subsec_nanos() as f64 / 1e9,
                &tz,
            ),
        )
    }

    /// Create a Julian Date from a UTC datetime string (ISO 8601 / RFC 3339).
    ///
    /// Args:
    ///     utc_str: UTC datetime string (e.g. "2000-01-01T12:00:00Z").
    ///
    /// Returns:
    ///     JulianDate: corresponding Julian Date.
    #[staticmethod]
    fn from_utc(utc_str: &str) -> PyResult<Self> {
        use chrono::{DateTime, Utc};
        let dt: DateTime<Utc> = utc_str.parse::<DateTime<Utc>>().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid UTC datetime: {e}"))
        })?;
        Ok(Self {
            inner: Time::<JD>::from_utc(dt),
        })
    }

    /// Create a Julian Date from a Python `datetime.datetime` object.
    ///
    /// Args:
    ///     dt: Python datetime object (must have timezone info).
    #[staticmethod]
    fn from_datetime(dt: &Bound<'_, PyAny>) -> PyResult<Self> {
        let timestamp: f64 = dt.call_method0("timestamp")?.extract()?;
        let chrono_dt = chrono::DateTime::<chrono::Utc>::from_timestamp(
            timestamp.floor() as i64,
            ((timestamp.fract()) * 1e9) as u32,
        )
        .ok_or_else(utc_conversion_failed)?;
        Ok(Self {
            inner: Time::<JD>::from_utc(chrono_dt),
        })
    }

    /// Julian centuries since J2000.0.
    fn julian_centuries(&self) -> f64 {
        self.inner.julian_centuries().value()
    }

    /// Julian years since J2000.0.
    fn julian_years(&self) -> f64 {
        self.inner.julian_years().value()
    }

    /// Julian millennia since J2000.0.
    fn julian_millennia(&self) -> f64 {
        self.inner.julian_millennias().value()
    }

    /// Add days to this Julian Date.
    ///
    /// Args:
    ///     days: number of days to add.
    ///
    /// Returns:
    ///     JulianDate: a new Julian Date offset by the given days.
    fn add_days(&self, days: f64) -> Self {
        Self {
            inner: self.inner + qtty::Days::new(days),
        }
    }

    /// Difference in days between this and another Julian Date.
    ///
    /// Args:
    ///     other: another Julian Date.
    ///
    /// Returns:
    ///     float: difference in days (self - other).
    fn difference(&self, other: &PyJulianDate) -> f64 {
        (self.inner - other.inner).value()
    }

    fn __add__(&self, days: f64) -> Self {
        self.add_days(days)
    }

    fn __radd__(&self, days: f64) -> Self {
        self.add_days(days)
    }

    fn __sub__<'py>(&self, other: &Bound<'py, PyAny>) -> PyResult<Py<PyAny>> {
        let py = other.py();
        if let Ok(other_jd) = other.extract::<PyJulianDate>() {
            Ok(self
                .difference(&other_jd)
                .into_pyobject(py)?
                .into_any()
                .unbind())
        } else if let Ok(days) = other.extract::<f64>() {
            Ok(self.add_days(-days).into_pyobject(py)?.into_any().unbind())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Can only subtract a float (days) or another JulianDate",
            ))
        }
    }

    fn __eq__(&self, other: &PyJulianDate) -> bool {
        self.inner == other.inner
    }

    fn __ne__(&self, other: &PyJulianDate) -> bool {
        self.inner != other.inner
    }

    fn __lt__(&self, other: &PyJulianDate) -> bool {
        self.inner < other.inner
    }

    fn __le__(&self, other: &PyJulianDate) -> bool {
        self.inner <= other.inner
    }

    fn __gt__(&self, other: &PyJulianDate) -> bool {
        self.inner > other.inner
    }

    fn __ge__(&self, other: &PyJulianDate) -> bool {
        self.inner >= other.inner
    }

    fn __repr__(&self) -> String {
        format!("JulianDate({})", self.inner.value())
    }

    fn __str__(&self) -> String {
        format!("JulianDate({})", self.inner.value())
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner.value().to_bits().hash(&mut hasher);
        hasher.finish()
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64,))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((cls, (self.inner.value(),)))
    }

    fn __float__(&self) -> f64 {
        self.inner.value()
    }
}
