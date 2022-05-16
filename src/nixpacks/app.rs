use anyhow::anyhow;
use std::path::Path;
use std::{env, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use globset::Glob;
use regex::Regex;
use serde::de::DeserializeOwned;
use walkdir::WalkDir;

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

    pub fn find_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = self.source.join(pattern);

        let pattern_str = match full_pattern.to_str() {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        let walker = WalkDir::new(&self.source);
        let glob = Glob::new(pattern_str)?.compile_matcher();

        let relative_paths = walker
            .sort_by_file_name()
            .into_iter()
            .filter_map(|result| result.ok()) // remove bad ones
            .map(|dir| dir.into_path()) // convert to paths
            .filter(|path| glob.is_match(path)) // find matches
            .collect();

        Ok(relative_paths)
    }

    pub fn has_match(&self, pattern: &str) -> bool {
        match self.find_files(pattern) {
            Ok(v) => !v.is_empty(),
            Err(_e) => false,
        }
    }

    pub fn read_file(&self, name: &str) -> Result<String> {
        fs::read_to_string(self.source.join(name)).map_err(|e| anyhow!(e))
    }

    pub fn find_match(&self, re: &Regex, pattern: &str) -> Result<bool> {
        let paths = match self.find_files(pattern) {
            Ok(v) => v,
            Err(_e) => return Ok(false),
        };

        for path in paths {
            let path_buf = fs::canonicalize(path)?;

            if let Some(p) = path_buf.to_str() {
                let f = self.read_file(p)?;
                if re.find(f.as_str()).is_some() {
                    return Ok(true);
                }
            }
        }

        Ok(false)
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

    pub fn read_yaml<T>(&self, name: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let contents = self.read_file(name)?;
        let yaml_file = serde_yaml::from_str(contents.as_str())?;
        Ok(yaml_file)
    }

    pub fn strip_source_path(&self, abs_path: &Path) -> Result<PathBuf> {
        let source_str = match self.source.to_str() {
            Some(s) => s,
            None => bail!("Failed to parse source path"),
        };

        // Strip source path from absolute path
        let stripped = match abs_path.strip_prefix(source_str) {
            Ok(p) => p,
            Err(_e) => abs_path,
        };

        // Convert path to PathBuf
        Ok(stripped.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{Map, Value};

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

    #[test]
    fn test_find_files() -> Result<()> {
        let app = App::new("./examples/monorepo")?;
        let m = app.find_files("**/*.tsx").unwrap();
        let dir = env::current_dir().unwrap();
        assert_eq!(
            m,
            vec![
                dir.join("examples/monorepo/packages/client/pages/_app.tsx")
                    .canonicalize()?,
                dir.join("examples/monorepo/packages/client/pages/index.tsx")
                    .canonicalize()?
            ]
        );
        Ok(())
    }

    #[test]
    fn test_find_match() -> Result<()> {
        let app = App::new("./examples/monorepo")?;
        let re = Regex::new(r"className")?;
        let m = app.find_match(&re, "**/*.tsx").unwrap();
        assert!(m);
        Ok(())
    }

    #[test]
    fn test_strip_source_path() -> Result<()> {
        let app = App::new("./examples/npm")?;
        let path_to_strip = app.source.join("foo/bar.txt");
        assert_eq!(
            &app.strip_source_path(&path_to_strip).unwrap(),
            Path::new("foo/bar.txt")
        );
        Ok(())
    }

    #[test]
    fn test_strip_source_path_no_source_prefix() -> Result<()> {
        let app = App::new("./examples/npm")?;
        assert_eq!(
            &app.strip_source_path(Path::new("no/prefix.txt"))?,
            Path::new("no/prefix.txt")
        );
        Ok(())
    }
}
