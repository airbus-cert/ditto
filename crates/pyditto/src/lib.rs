use ditto::impostor::apply_engine;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

struct PyDittoError(ditto::error::Error);

impl From<PyDittoError> for PyErr {
    fn from(err: PyDittoError) -> Self {
        PyErr::new::<PyRuntimeError, _>(err.0.to_string())
    }
}

#[pyfunction]
fn obfuscate_powershell(name: String, src: String) -> PyResult<String> {
    apply_engine(&name, &src).map_err(|e| PyErr::from(PyDittoError(e)).into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyditto(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(obfuscate_powershell, m)?)?;
    Ok(())
}
