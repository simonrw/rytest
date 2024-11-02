use std::path::{Path, PathBuf};

use color_eyre::eyre::{self, Context};
use tree_sitter::Node;

use crate::TestDefinition;

pub async fn collect_tests(root: impl AsRef<Path>) -> eyre::Result<Vec<TestDefinition>> {
    let root = root.as_ref();
    let test_files = find_test_files(root).await.wrap_err("finding test files")?;
    let mut out = Vec::new();
    for test_file in test_files {
        let tests = extract_tests_from_test_file(&test_file)
            .await
            .wrap_err_with(|| format!("extracting tests from {}", test_file.display()))?;
        out.extend(tests);
    }
    Ok(out)
}

struct Visitor {
    filename: PathBuf,
    bytes: Vec<u8>,
    tests: Vec<TestDefinition>,
}

impl Visitor {
    pub async fn new(filename: impl AsRef<Path>) -> eyre::Result<Self> {
        let filename = filename.as_ref();
        let bytes = tokio::fs::read(filename).await.wrap_err("reading file")?;
        Ok(Self {
            filename: filename.to_path_buf(),
            bytes,
            tests: Vec::new(),
        })
    }

    fn tests(self) -> Vec<TestDefinition> {
        self.tests
    }

    fn visit(&mut self) -> eyre::Result<()> {
        let language = tree_sitter_python::LANGUAGE;
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language.into())
            .wrap_err("configuring language")?;

        let tree = parser
            .parse(&self.bytes, None)
            .ok_or_else(|| eyre::eyre!("parsing file"))?;

        let root = tree.root_node();

        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            match child.kind() {
                "decorated_definition" => self.handle_decorated_definition(child, None)?,
                "class_definition" => self.handle_class_definition(child)?,
                "function_definition" => self.handle_function_definition(child, None)?,
                "import_statement"
                | "import_from_statement"
                | "expression_statement"
                | "comment"
                | "if_statement"
                | "try_statement"
                | "assert_statement" => continue,
                kind => todo!("{kind}"),
            }
        }

        Ok(())
    }

    fn handle_decorated_definition(
        &mut self,
        node: Node,
        class_name: Option<String>,
    ) -> eyre::Result<()> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    self.handle_function_definition(child, class_name.clone())?
                }
                "class_definition" => self.handle_class_definition(child)?,
                "decorator" | "comment" => continue,
                kind => todo!("{kind}"),
            }
        }
        Ok(())
    }

    fn handle_class_definition(&mut self, node: Node) -> eyre::Result<()> {
        // TODO: nested classes?
        let Some(class_name_node) = node.child(1) else {
            eyre::bail!("no class name found");
        };

        if class_name_node.kind() != "identifier" {
            eyre::bail!(
                "invalid class name node type, expected 'identifier', got '{}'",
                class_name_node.kind()
            );
        }

        // TODO: can we prevent this clone?
        let bytes = self.bytes.clone();
        let class_name = class_name_node
            .utf8_text(&bytes)
            .wrap_err("reading class name")?;

        if !class_name.starts_with("Test") {
            // stop parsing
            return Ok(());
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor).skip(2) {
            match child.kind() {
                "block" => self.handle_class_block(child, Some(class_name.to_string()))?,
                ":" | "argument_list" | "comment" => continue,
                kind => todo!("{kind}"),
            }
        }

        Ok(())
    }

    fn handle_class_block(&mut self, node: Node, class_name: Option<String>) -> eyre::Result<()> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "decorated_definition" => {
                    self.handle_decorated_definition(child, class_name.clone())?
                }
                "function_definition" => {
                    self.handle_function_definition(child, class_name.clone())?
                }
                "expression_statement" | "comment" => continue,
                kind => todo!("{kind}"),
            }
        }
        Ok(())
    }

    fn handle_function_definition(
        &mut self,
        node: Node,
        class_name: Option<String>,
    ) -> eyre::Result<()> {
        let Some(identifier_node) = node.child(1) else {
            eyre::bail!("no identifier node found");
        };

        let bytes = self.bytes.clone();
        let identifier = identifier_node
            .utf8_text(&bytes)
            .wrap_err("reading bytes for function identifier")?;

        if !identifier.starts_with("test_") {
            return Ok(());
        }

        self.emit(identifier, class_name)
            .wrap_err("sending test case")?;

        Ok(())
    }

    fn emit(
        &mut self,
        test_name: impl Into<String>,
        class_name: Option<String>,
    ) -> eyre::Result<()> {
        let test_case = TestDefinition {
            name: test_name.into(),
            path: self.filename.to_path_buf(),
            class_name,
        };

        self.tests.push(test_case);

        Ok(())
    }
}

async fn extract_tests_from_test_file(
    test_file: impl AsRef<Path>,
) -> eyre::Result<Vec<TestDefinition>> {
    let mut visitor = Visitor::new(test_file).await.wrap_err("creating visitor")?;
    visitor.visit().wrap_err("parsing file")?;
    Ok(visitor.tests())
}

async fn find_test_files(root: impl AsRef<Path>) -> eyre::Result<Vec<PathBuf>> {
    let root = root.as_ref();
    let mut out = Vec::new();
    ignore::Walk::new(root).for_each(|result| {
        if let Ok(entry) = result {
            if entry.path().is_file() {
                out.push(entry.path().to_path_buf());
            }
        }
    });
    Ok(out)
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
                ..Default::default()
            },]
        );
        Ok(())
    }

    #[tokio::test]
    async fn class_method() -> eyre::Result<()> {
        let test_dir = tempfile::tempdir().wrap_err("creating temporary directory")?;
        let file_contents = r#"
class TestClass:
    def test_method(self):
        assert 1 == 2
    "#;
        let test_file = test_dir.path().join("test_method.py");
        tokio::fs::write(&test_file, file_contents)
            .await
            .wrap_err("writing test file contents")?;

        let tests = collect_tests(&test_dir)
            .await
            .wrap_err("collecting tests")?;
        assert_eq!(
            tests,
            vec![TestDefinition {
                name: "test_method".to_string(),
                path: test_dir.path().join("test_method.py"),
                class_name: Some("TestClass".to_string()),
            },]
        );
        Ok(())
    }
}
