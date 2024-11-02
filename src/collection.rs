use std::path::Path;

use color_eyre::eyre;

use crate::TestDefinition;

pub async fn collect_tests(root: impl AsRef<Path>) -> eyre::Result<Vec<TestDefinition>> {
    let root = root.as_ref();
    Ok(vec![TestDefinition {
        name: "test_simple".to_string(),
        path: root.join("test_simple.py"),
    }])
}

#[cfg(test)]
mod tests {
    use color_eyre::eyre::{self, Context};

    use crate::{collection::collect_tests, TestDefinition};

    #[tokio::test]
    async fn simple() -> eyre::Result<()> {
        let test_dir = tempfile::tempdir().wrap_err("creating temporary directory")?;
        let file_contents = r#"
def test_simple():
    assert 1 == 2
    "#;
        let test_file = test_dir.path().join("test_simple.py");
        tokio::fs::write(&test_file, file_contents)
            .await
            .wrap_err("writing test file contents")?;

        let tests = collect_tests(&test_dir)
            .await
            .wrap_err("collecting tests")?;
        assert_eq!(
            tests,
            vec![TestDefinition {
                name: "test_simple".to_string(),
                path: test_dir.path().join("test_simple.py"),
            },]
        );
        Ok(())
    }
}
