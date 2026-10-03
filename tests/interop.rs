use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use tempoch::qtty::Second;
use tempoch::{Period, Time, UTC};
use tempoch_py::interop::{datetime_to_time, period_to_datetimes, time_to_datetime};

fn utc_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .unwrap_or_else(|| panic!("valid test datetime"))
}

#[test]
fn aware_utc_roundtrip_preserves_subseconds() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        let datetime = py.import("datetime")?;
        let input = datetime.getattr("datetime")?.call1((
            2024,
            6,
            21,
            12,
            34,
            56,
            123_456,
            datetime.getattr("timezone")?.getattr("utc")?,
        ))?;

        let instant = datetime_to_time(&input)?;
        let output = time_to_datetime(py, instant)?;

        let actual = output.extract::<DateTime<FixedOffset>>()?;
        let expected = input.extract::<DateTime<FixedOffset>>()?;
        assert!((actual.timestamp_micros() - expected.timestamp_micros()).abs() < 50);
        assert_eq!(actual.offset().local_minus_utc(), 0);
        assert!(actual.timestamp_subsec_micros() > 123_400);
        Ok(())
    })
}

#[test]
fn positive_and_negative_offsets_normalize_to_utc() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        let datetime = py.import("datetime")?;
        let datetime_type = datetime.getattr("datetime")?;
        let timezone = datetime.getattr("timezone")?;
        let timedelta = datetime.getattr("timedelta")?;

        let positive_tz = timezone.call1((
            timedelta.call((), Some(&[("hours", 5), ("minutes", 30)].into_py_dict(py)?))?,
        ))?;
        let positive = datetime_type.call1((2024, 1, 2, 17, 30, 0, 0, positive_tz))?;
        let positive_output = time_to_datetime(py, datetime_to_time(&positive)?)?;
        let expected = datetime_type.call1((2024, 1, 2, 12, 0, 0, 0, timezone.getattr("utc")?))?;
        let expected_chrono = expected.extract::<DateTime<FixedOffset>>()?;
        let positive_chrono = positive_output.extract::<DateTime<FixedOffset>>()?;
        assert!(
            (positive_chrono.timestamp_micros() - expected_chrono.timestamp_micros()).abs() < 50
        );

        let negative_tz =
            timezone.call1((timedelta.call((), Some(&[("hours", -4)].into_py_dict(py)?))?,))?;
        let negative = datetime_type.call1((2024, 1, 2, 8, 0, 0, 0, negative_tz))?;
        let negative_output = time_to_datetime(py, datetime_to_time(&negative)?)?;
        let negative_chrono = negative_output.extract::<DateTime<FixedOffset>>()?;
        assert!(
            (negative_chrono.timestamp_micros() - expected_chrono.timestamp_micros()).abs() < 50
        );
        assert_eq!(negative_chrono.offset().local_minus_utc(), 0);
        Ok(())
    })
}

#[test]
fn naive_and_non_datetime_inputs_are_rejected() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        let datetime = py.import("datetime")?;
        let naive = datetime
            .getattr("datetime")?
            .call1((2024, 1, 2, 12, 0, 0))?;
        let naive_error = datetime_to_time(&naive).expect_err("naive datetime must fail");
        assert!(naive_error.is_instance_of::<pyo3::exceptions::PyValueError>(py));

        let not_datetime = "2024-01-02T12:00:00Z".into_pyobject(py)?.into_any();
        let type_error = datetime_to_time(&not_datetime).expect_err("string must fail");
        assert!(type_error.is_instance_of::<pyo3::exceptions::PyTypeError>(py));
        Ok(())
    })
}

#[test]
fn normal_time_and_period_convert_to_python_datetimes() -> PyResult<()> {
    Python::initialize();
    let start_chrono = utc_datetime(2024, 1, 2, 12, 0, 0);
    let end_chrono = utc_datetime(2024, 1, 2, 13, 0, 0);
    let start = Time::<UTC>::try_from_chrono(start_chrono)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
    let end = Time::<UTC>::try_from_chrono(end_chrono)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;

    Python::attach(|py| {
        let single = time_to_datetime(py, start)?;
        assert_eq!(single.getattr("hour")?.extract::<u8>()?, 12);

        let (py_start, py_end) = period_to_datetimes(py, Period::new(start, end))?;
        assert_eq!(py_start.getattr("hour")?.extract::<u8>()?, 12);
        assert_eq!(py_end.getattr("hour")?.extract::<u8>()?, 13);
        assert!(py_start.lt(&py_end)?);
        Ok(())
    })
}

#[test]
fn out_of_chrono_range_returns_python_exception() -> PyResult<()> {
    Python::initialize();
    let value = Time::<UTC>::from_raw_j2000_seconds(Second::new(1.0e20))
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
    Python::attach(|py| {
        let error = time_to_datetime(py, value).expect_err("huge instant must fail");
        assert!(error.is_instance_of::<pyo3::exceptions::PyValueError>(py));
        Ok(())
    })
}
