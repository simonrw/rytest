use clap::Parser;
use collection::collect_items;
use pyo3::prelude::*;
use tokio::runtime;

mod collection;

use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    path: PathBuf,
}

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

    // Command line arguments start with the Python interpreter
    let args = Args::parse_from(std::env::args().skip(1));

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let items = runtime.block_on(async move { collect_items(args.path).await.unwrap() });
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
