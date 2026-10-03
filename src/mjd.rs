//! Python wrappers for `Time<MJD>` (Modified Julian Date).

use pyo3::prelude::*;
use pyo3::Py;
use tempoch::qtty::Day;
use tempoch::{ModifiedJulianDate, JD, TT, UTC};

use crate::errors::{ensure_finite, map_conversion_error};
use crate::interop;
use crate::jd::PyJulianDate;

/// A Modified Julian Date — JD minus 2,400,000.5.
///
/// Wraps the Rust `Time<MJD>` type for Python with full arithmetic,
/// comparison, UTC conversion, and scale-conversion support.
///
/// Examples:
///     >>> from tempoch import ModifiedJulianDate
///     >>> mjd = ModifiedJulianDate(51544.5)
///     >>> print(mjd.to_jd())
///     JulianDate(2451545.0)
#[pyclass(name = "ModifiedJulianDate", module = "tempoch", from_py_object)]
#[derive(Clone, Copy)]
pub struct PyModifiedJulianDate {
    pub(crate) inner: ModifiedJulianDate<TT>,
}

impl PyModifiedJulianDate {
    /// Create from an inner Rust `ModifiedJulianDate`.
    pub fn from_inner(inner: ModifiedJulianDate<TT>) -> Self {
        Self { inner }
    }
}

#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyModifiedJulianDate {
    /// Create a new Modified Julian Date from a day number.
    ///
    /// Args:
    ///     value: Modified Julian Day number.
    ///
    /// Raises:
    ///     NonFiniteTimeError: if the value is NaN or infinite.
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        ensure_finite(value)?;
        Ok(Self {
            inner: ModifiedJulianDate::<TT>::new(value),
        })
    }

    /// The raw MJD day-number value.
    #[getter]
    fn value(&self) -> f64 {
        self.inner.value()
    }

    /// Convert to Julian Date.
    fn to_jd(&self) -> PyJulianDate {
        PyJulianDate::from_inner(self.inner.to::<JD>())
    }

    /// Convert to a UTC datetime string (ISO 8601).
    ///
    /// Returns:
    ///     str: UTC datetime in ISO 8601 format.
    ///
    /// Raises:
    ///     ConversionError: if the value is outside chrono's representable range.
    fn to_utc(&self) -> PyResult<String> {
        let dt = self
            .inner
            .to::<UTC>()
            .try_to_chrono()
            .map_err(map_conversion_error)?;
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
        interop::time_to_datetime(py, self.inner.to::<UTC>().to_j2000s())
    }

    /// Create a Modified Julian Date from a UTC datetime string.
    ///
    /// Args:
    ///     utc_str: UTC datetime string (e.g. "2000-01-01T12:00:00Z").
    #[staticmethod]
    fn from_utc(utc_str: &str) -> PyResult<Self> {
        use chrono::{DateTime, FixedOffset, Utc};
        let dt: DateTime<FixedOffset> = utc_str.parse().map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid UTC datetime: {e}"))
        })?;
        let utc = tempoch::Time::<UTC>::try_from_chrono(dt.with_timezone(&Utc))
            .map_err(map_conversion_error)?;
        Ok(Self {
            inner: utc.to::<TT>().to::<tempoch::MJD>(),
        })
    }

    /// Create a Modified Julian Date from a Python `datetime.datetime` object.
    #[staticmethod]
    fn from_datetime(dt: &Bound<'_, PyAny>) -> PyResult<Self> {
        let utc = interop::datetime_to_time(dt)?;
        Ok(Self {
            inner: utc.to::<TT>().to::<tempoch::MJD>(),
        })
    }

    /// Add days to this Modified Julian Date.
    fn add_days(&self, days: f64) -> Self {
        Self {
            inner: self.inner + Day::new(days),
        }
    }

    /// Difference in days between this and another Modified Julian Date.
    fn difference(&self, other: &PyModifiedJulianDate) -> f64 {
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
        if let Ok(other_mjd) = other.extract::<PyModifiedJulianDate>() {
            Ok(self
                .difference(&other_mjd)
                .into_pyobject(py)?
                .into_any()
                .unbind())
        } else if let Ok(days) = other.extract::<f64>() {
            Ok(self.add_days(-days).into_pyobject(py)?.into_any().unbind())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "Can only subtract a float (days) or another ModifiedJulianDate",
            ))
        }
    }

    fn __eq__(&self, other: &PyModifiedJulianDate) -> bool {
        self.inner == other.inner
    }

    fn __ne__(&self, other: &PyModifiedJulianDate) -> bool {
        self.inner != other.inner
    }

    fn __lt__(&self, other: &PyModifiedJulianDate) -> bool {
        self.inner < other.inner
    }

    fn __le__(&self, other: &PyModifiedJulianDate) -> bool {
        self.inner <= other.inner
    }

    fn __gt__(&self, other: &PyModifiedJulianDate) -> bool {
        self.inner > other.inner
    }

    fn __ge__(&self, other: &PyModifiedJulianDate) -> bool {
        self.inner >= other.inner
    }

    fn __repr__(&self) -> String {
        format!("ModifiedJulianDate({})", self.inner.value())
    }

    fn __str__(&self) -> String {
        format!("ModifiedJulianDate({})", self.inner.value())
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
