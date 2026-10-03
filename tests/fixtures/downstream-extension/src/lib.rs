use pyo3::prelude::*;
use tempoch::Time;
use tempoch_py::interop;

#[pyfunction]
fn roundtrip<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let canonical: Time<tempoch::UTC> = interop::datetime_to_time(value)?;
    interop::time_to_datetime(value.py(), canonical)
}

#[pymodule]
fn _tempoch_downstream(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(roundtrip, module)?)
}
