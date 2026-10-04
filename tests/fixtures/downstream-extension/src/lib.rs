use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use tempoch::{Period, Time, TimeContext, UTC};
use tempoch_py::interop;

#[pyfunction]
fn roundtrip<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let context = TimeContext::new();
    let canonical: Time<UTC> = interop::datetime_to_time_with(value, &context)?;
    interop::time_to_datetime_with(value.py(), canonical, &context)
}

#[pyfunction]
fn period_roundtrip<'py>(
    start: &Bound<'py, PyAny>,
    end: &Bound<'py, PyAny>,
) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
    let context = TimeContext::new();
    let start_time = interop::datetime_to_time_with(start, &context)?;
    let end_time = interop::datetime_to_time_with(end, &context)?;
    let period = Period::try_new(start_time, end_time)
        .map_err(|_| PyValueError::new_err("period start must not be after end"))?;

    let _default_context = interop::period_to_datetimes(start.py(), period)?;
    interop::period_to_datetimes_with(start.py(), period, &context)
}

#[pymodule]
fn _tempoch_downstream(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(roundtrip, module)?)?;
    module.add_function(wrap_pyfunction!(period_roundtrip, module)?)?;
    Ok(())
}
