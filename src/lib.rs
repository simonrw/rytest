use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn cli_main() -> PyResult<()> {
    Python::with_gil(|py| {
        let pytest = py.import_bound("pytest")?;
        pytest.call_method0("console_main")?;
        Ok(())
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn rytest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    Ok(())
}
