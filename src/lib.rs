mod collection;

use std::path::PathBuf;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct TestDefinition {
    pub path: PathBuf,
    pub class_name: Option<String>,
    pub name: String,
}
