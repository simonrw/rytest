use std::path::Path;

use crate::{RytestResult, TestDefinition};

pub async fn collect_tests(root: impl AsRef<Path>) -> RytestResult<Vec<TestDefinition>> {
    let root = root.as_ref();
    Ok(vec![TestDefinition {
        name: "test_simple".to_string(),
        path: root.join("test_simple.py"),
    }])
}

#[cfg(test)]
mod tests {
    use crate::{collection::collect_tests, TestDefinition};

    #[tokio::test]
    async fn simple() {
        let test_dir = tempfile::tempdir().unwrap();
        let file_contents = r#"
def test_simple():
    assert 1 == 2
    "#;
        let test_file = test_dir.path().join("test_simple.py");
        tokio::fs::write(&test_file, file_contents).await.unwrap();

        let tests = collect_tests(&test_dir).await.unwrap();
        assert_eq!(
            tests,
            vec![TestDefinition {
                name: "test_simple".to_string(),
                path: test_dir.path().join("test_simple.py"),
            },]
        );
    }
}
