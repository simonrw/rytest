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

#[derive(Default, PartialEq, Eq)]
pub struct TestDefinition {
    pub path: PathBuf,
    pub class_name: Option<String>,
    pub name: String,
    pub fixture_names: Vec<String>,
}

impl std::fmt::Debug for TestDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(class_name) = &self.class_name {
            if self.fixture_names.is_empty() {
                write!(
                    f,
                    "{path}::{class_name}::{name} (uses no fixtures)",
                    path = self.path.display(),
                    class_name = class_name,
                    name = self.name,
                )
            } else {
                write!(
                    f,
                    "{path}::{class_name}::{name} (uses {fixture_names})",
                    path = self.path.display(),
                    class_name = class_name,
                    name = self.name,
                    fixture_names = self.fixture_names.join(", ")
                )
            }
        } else if self.fixture_names.is_empty() {
            write!(
                f,
                "{path}::{name} (uses no fixtures)",
                path = self.path.display(),
                name = self.name,
            )
        } else {
            write!(
                f,
                "{path}::{name} (uses {fixture_names})",
                path = self.path.display(),
                name = self.name,
                fixture_names = self.fixture_names.join(", ")
            )
        }
    }
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
    for test in items.tests {
        println!("test: {test:?}");
    }

    for fixture in items.fixtures {
        println!("fixture: {fixture:?}");
    }
    Ok(())
}

#[pymodule]
fn rytest(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    Ok(())
}
