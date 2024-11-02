mod collection;

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum RytestError {}

pub type RytestResult<T> = std::result::Result<T, RytestError>;

#[derive(Debug, PartialEq, Eq)]
pub struct TestDefinition {
    pub path: PathBuf,
    pub name: String,
}
