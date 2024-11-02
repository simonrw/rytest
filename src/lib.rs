use collection::collect_items;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tokio::runtime;

mod collection;

use std::{env::current_dir, path::PathBuf};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct TestDefinition {
    pub path: PathBuf,
    pub class_name: Option<String>,
    pub name: String,
    pub fixture_names: Vec<String>,
}

// Entrypoint to the Rust world
#[pyfunction]
fn cli_main() -> PyResult<()> {
    tracing_subscriber::fmt::init();

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let root = current_dir().map_err(|e| PyRuntimeError::new_err(format!("{e}")))?;
    let items = runtime.block_on(async move { collect_items(root).await.unwrap() });
    for item in items {
        println!("{item:?}");
    }
    Ok(())
}

#[pymodule]
fn rytest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    Ok(())
}
