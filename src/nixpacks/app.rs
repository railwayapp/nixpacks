use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use glob::glob;
use regex::Regex;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct App {
    pub source: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl App {
    pub fn new(path: &str) -> Result<App> {
        let current_dir = env::current_dir()?;
        let source = current_dir
            .join(path)
            .canonicalize()
            .context("Failed to read app source directory")?;

        let dir = fs::read_dir(source.clone()).context("Failed to read app source directory")?;
        let paths: Vec<PathBuf> = dir.map(|path| path.unwrap().path()).collect();

        Ok(App { source, paths })
    }

    pub fn includes_file(&self, name: &str) -> bool {
        fs::canonicalize(self.source.join(name)).is_ok()
    }

    pub fn read_file(&self, name: &str) -> Result<String> {
        let name = self.source.join(name);
        let contents = fs::read_to_string(name)?;
        Ok(contents)
    }

    pub fn find_match(&self, re: &Regex, pattern: &str) -> bool {
        let full_pattern = self.source.join(pattern);
        let entries = glob(full_pattern.to_str().unwrap()).expect("Failed to read glob pattern");

        let mut matched = false;
        for entry in entries {
            let path_buf = fs::canonicalize(entry.unwrap()).unwrap();
            let f = self.read_file(path_buf.to_str().unwrap()).unwrap();
            if re.find(f.as_str()).is_some() {
                matched = true;
                break;
            }
        }
        matched
    }

    pub fn read_json<T>(&self, name: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let contents = self.read_file(name)?;
        let value: T = serde_json::from_str(contents.as_str())?;
        Ok(value)
    }

    pub fn read_toml<T>(&self, name: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let contents = self.read_file(name)?;
        let toml_file = toml::from_str(contents.as_str())?;
        Ok(toml_file)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use serde_json::{Map, Value};

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct TestPackageJson {
        name: String,
        scripts: HashMap<String, String>,
    }

    #[test]
    fn test_creates_app() -> Result<()> {
        let app = App::new("./examples/npm")?;
        assert_eq!(app.paths.len(), 5);
        Ok(())
    }

    #[test]
    fn test_read_file() -> Result<()> {
        let app = App::new("./examples/npm")?;
        assert_eq!(
            app.read_file("index.ts")?.trim_end(),
            "console.log(\"Hello from NPM\");"
        );
        Ok(())
    }

    #[test]
    fn test_read_json_file() -> Result<()> {
        let app = App::new("./examples/npm")?;
        let value: Map<String, Value> = app.read_json("package.json")?;
        assert!(value.get("name").is_some());
        assert_eq!(value.get("name").unwrap(), "npm");
        Ok(())
    }

    #[test]
    fn test_read_structured_json_file() -> Result<()> {
        let app = App::new("./examples/npm")?;
        let value: TestPackageJson = app.read_json("package.json")?;
        assert_eq!(value.name, "npm");
        assert_eq!(value.scripts.get("build").unwrap(), "tsc -p tsconfig.json");
        Ok(())
    }

    #[test]
    fn test_read_toml_file() -> Result<()> {
        let app = App::new("./examples/rust-rocket")?;
        let toml_file: toml::Value = app.read_toml("Cargo.toml")?;
        assert!(toml_file.get("package").is_some());
        assert_eq!(
            toml_file
                .get("package")
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "rocket"
        );
        Ok(())
    }
}
