//! Reusable PyO3 interoperability for Python datetimes and tempoch UTC values.
//!
//! Python inputs must be timezone-aware. Fixed offsets are normalized to UTC,
//! and all scientific time handling is delegated to [`tempoch`].
//!
//! ```no_run
//! # use pyo3::prelude::*;
//! # fn roundtrip<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
//! let instant = tempoch_py::interop::datetime_to_time(value)?;
//! tempoch_py::interop::time_to_datetime(value.py(), instant)
//! # }
//! ```

use chrono::{DateTime, FixedOffset, Utc};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDateTime;
use tempoch::{Period, Time, TimeContext, UTC};

use crate::errors::map_conversion_error;

fn aware_datetime(value: &Bound<'_, PyAny>) -> PyResult<DateTime<FixedOffset>> {
    if !value.is_instance_of::<PyDateTime>() {
        return Err(PyTypeError::new_err(
            "value must be a timezone-aware datetime.datetime",
        ));
    }

    if value.call_method0("utcoffset")?.is_none() {
        return Err(PyValueError::new_err(
            "naive datetimes are not accepted; value must be timezone-aware",
        ));
    }

    value.extract::<DateTime<FixedOffset>>().map_err(|error| {
        PyValueError::new_err(format!(
            "value must be a valid timezone-aware datetime.datetime: {error}"
        ))
    })
}

/// Convert a timezone-aware Python `datetime.datetime` to canonical tempoch UTC.
///
/// Non-UTC offsets are normalized to UTC. Naive datetimes and non-datetime
/// objects are rejected with Python exceptions.
pub fn datetime_to_time(value: &Bound<'_, PyAny>) -> PyResult<Time<UTC>> {
    datetime_to_time_with(value, &TimeContext::new())
}

/// Convert with an explicit tempoch [`TimeContext`].
pub fn datetime_to_time_with(
    value: &Bound<'_, PyAny>,
    context: &TimeContext,
) -> PyResult<Time<UTC>> {
    let datetime = aware_datetime(value)?.with_timezone(&Utc);
    Time::<UTC>::try_from_chrono_with(datetime, context).map_err(map_conversion_error)
}

/// Convert canonical tempoch UTC to a timezone-aware UTC Python datetime.
pub fn time_to_datetime<'py>(py: Python<'py>, value: Time<UTC>) -> PyResult<Bound<'py, PyAny>> {
    time_to_datetime_with(py, value, &TimeContext::new())
}

/// Convert to a Python datetime with an explicit tempoch [`TimeContext`].
pub fn time_to_datetime_with<'py>(
    py: Python<'py>,
    value: Time<UTC>,
    context: &TimeContext,
) -> PyResult<Bound<'py, PyAny>> {
    let datetime = value
        .try_to_chrono_with(context)
        .map_err(map_conversion_error)?;
    Ok(datetime.into_pyobject(py)?.into_any())
}

/// Convert a canonical tempoch UTC period to Python start/end datetimes.
pub fn period_to_datetimes<'py>(
    py: Python<'py>,
    period: Period<UTC>,
) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
    period_to_datetimes_with(py, period, &TimeContext::new())
}

/// Convert a UTC period with an explicit tempoch [`TimeContext`].
pub fn period_to_datetimes_with<'py>(
    py: Python<'py>,
    period: Period<UTC>,
    context: &TimeContext,
) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
    Ok((
        time_to_datetime_with(py, period.start, context)?,
        time_to_datetime_with(py, period.end, context)?,
    ))
}
