mod collection;

use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub struct TestDefinition {
    pub path: PathBuf,
    pub name: String,
}
