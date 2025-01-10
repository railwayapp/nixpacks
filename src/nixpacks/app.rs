use path_slash::PathBufExt;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;
use std::{env, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use globset::Glob;
use ignore::{DirEntry, WalkBuilder};
use regex::Regex;
use serde::de::DeserializeOwned;

pub type StaticAssets = BTreeMap<String, String>;

pub const ASSETS_DIR: &str = "/assets/";

/// Represents a project's file and directory paths.
#[derive(Debug, Clone)]
pub struct App {
    pub source: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl App {
    /// Generate a path representation of a project.
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

    /// Check if a file exists
    pub fn includes_file(&self, name: &str) -> bool {
        self.source.join(name).is_file()
    }

    /// Returns a list of file paths matching a glob pattern
    ///
    /// # Errors
    /// Creating the Glob fails
    pub fn find_files(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let directories = self
            .find_glob(pattern)?
            .into_iter()
            .filter(|path| path.is_file())
            .collect();

        Ok(directories)
    }

    /// Returns a list of directory paths matching a glob pattern
    ///
    /// # Errors
    /// Creating the Glob fails
    pub fn find_directories(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let directories = self
            .find_glob(pattern)?
            .into_iter()
            .filter(|path| path.is_dir())
            .collect();

        Ok(directories)
    }

    /// Check whether a shell-style pattern matches any paths in the app.
    fn find_glob(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let full_pattern = self.source.join(pattern);

        let pattern_str = match full_pattern.to_str() {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        let walker = WalkBuilder::new(&self.source)
            // this includes hidden directories & files
            .hidden(false)
            .sort_by_file_name(OsStr::cmp)
            .build();
        let glob = Glob::new(pattern_str)?.compile_matcher();

        let relative_paths = walker
            .into_iter()
            .filter_map(Result::ok) // remove bad ones
            .map(DirEntry::into_path) // convert to paths
            .filter(|path| glob.is_match(path)) // find matches
            .collect();

        Ok(relative_paths)
    }

    /// Check if a path matching a glob exists
    pub fn has_match(&self, pattern: &str) -> bool {
        match self.find_files(pattern) {
            Ok(v) => !v.is_empty(),
            Err(_e) => false,
        }
    }

    /// Read the contents of a file
    ///
    /// # Errors
    /// This will error if the path doesn't exist, or if the contents isn't UTF-8
    pub fn read_file(&self, name: &str) -> Result<String> {
        let data = fs::read_to_string(PathBuf::from_slash_lossy(
            self.source.join(name).as_os_str(),
        ))
        .with_context(|| {
            let relative_path = self.strip_source_path(Path::new(name)).unwrap();
            format!("Error reading {}", relative_path.to_str().unwrap())
        })?;

        Ok(data.replace("\r\n", "\n"))
    }

    /// Check whether filenames matching a pattern exist in the project.
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

    /// Check if a directory exists
    pub fn includes_directory(&self, name: &str) -> bool {
        self.source.join(name).is_dir()
    }

    #[cfg(target_os = "windows")]
    pub fn is_file_executable(&self, name: &str) -> bool {
        true
    }

    /// Check if a path is an executable file
    #[cfg(not(target_os = "windows"))]
    pub fn is_file_executable(&self, name: &str) -> bool {
        use std::os::unix::prelude::PermissionsExt;

        let path = self.source.join(name);
        if path.is_file() {
            let metadata = path.metadata().unwrap();
            metadata.permissions().mode() & 0o111 != 0
        } else {
            false
        }
    }

    /// Try to json-parse a file.
    pub fn read_json<T>(&self, name: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let contents = self.read_file(name)?;
        let value: T = serde_json::from_str(contents.as_str()).with_context(|| {
            let relative_path = self.strip_source_path(Path::new(name)).unwrap();
            format!("Error reading {} as JSON", relative_path.to_str().unwrap())
        })?;
        Ok(value)
    }

    /// Try to toml-parse a file.
    pub fn read_toml<T>(&self, name: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let contents = self.read_file(name)?;
        let toml_file = toml::from_str(contents.as_str()).with_context(|| {
            let relative_path = self.strip_source_path(Path::new(name)).unwrap();
            format!("Error reading {} as TOML", relative_path.to_str().unwrap())
        })?;
        Ok(toml_file)
    }

    /// Parse jsonc files as json by ignoring all kinds of comments
    pub fn read_jsonc<T>(&self, name: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut cleaned_jsonc = String::new();
        let contents = self.read_file(name)?;
        let mut chars = contents.chars().peekable();
        while let Some(current_char) = chars.next() {
            match current_char {
                '/' if chars.peek() == Some(&'/') => {
                    while let Some(&next_char) = chars.peek() {
                        chars.next();
                        if next_char == '\n' {
                            break;
                        }
                    }
                }
                '/' if chars.peek() == Some(&'*') => {
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('*') if chars.peek() == Some(&'/') => {
                                chars.next();
                                break;
                            }
                            None => break,
                            _ => continue,
                        }
                    }
                }
                _ => cleaned_jsonc.push(current_char),
            }
        }
        let value: T = serde_json::from_str(cleaned_jsonc.as_str()).with_context(|| {
            let relative_path = self.strip_source_path(Path::new(name)).unwrap();
            format!("Error reading {} as JSONC", relative_path.to_str().unwrap())
        })?;
        Ok(value)
    }

    /// Try to yaml-parse a file.
    pub fn read_yaml<T>(&self, name: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let contents = self.read_file(name)?;
        let yaml_file = serde_yaml::from_str(contents.as_str())?;
        Ok(yaml_file)
    }

    /// Convert an absolute path to a path relative to the app source directory
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

    /// Get the path in the container to an asset defined in `static_assets`.
    pub fn asset_path(&self, name: &str) -> String {
        format!("{ASSETS_DIR}{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};
    use serde_json::{Map, Value};
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize)]
    struct TestPackageJson {
        name: String,
        scripts: HashMap<String, String>,
    }

    #[test]
    fn test_creates_app() -> Result<()> {
        let app = App::new("./examples/node-npm")?;
        assert_eq!(app.paths.len(), 5);
        Ok(())
    }

    #[test]
    fn test_read_file() -> Result<()> {
        let app = App::new("./examples/node-npm")?;
        assert_eq!(
            app.read_file("index.ts")?.trim_end(),
            "console.log(\"Hello from NPM\");"
        );
        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_is_file_executable() -> Result<()> {
        let app = App::new("./examples/java-gradle-hello-world")?;
        assert!(app.is_file_executable("gradlew"));
        assert!(!app.is_file_executable("build.gradle"));
        Ok(())
    }

    #[test]
    fn test_read_json_file() -> Result<()> {
        let app = App::new("./examples/node-npm")?;
        let value: Map<String, Value> = app.read_json("package.json")?;
        assert!(value.get("name").is_some());
        assert_eq!(value.get("name").unwrap(), "npm");
        Ok(())
    }

    #[test]
    fn test_read_structured_json_file() -> Result<()> {
        let app = App::new("./examples/node-npm")?;
        let value: TestPackageJson = app.read_json("package.json")?;
        assert_eq!(value.name, "npm");
        assert_eq!(value.scripts.get("build").unwrap(), "tsc -p tsconfig.json");
        Ok(())
    }

    #[test]
    fn test_read_jsonc_file() -> Result<()> {
        let app = App::new("./examples/deno-jsonc")?;
        let value: Map<String, Value> = app.read_jsonc("deno.jsonc")?;
        assert!(value.get("tasks").is_some());
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
        let app = App::new("./examples/node-monorepo")?;
        let m = app.find_files("**/*.tsx").unwrap();
        let dir = env::current_dir().unwrap();
        assert_eq!(
            m,
            vec![
                dir.join("examples/node-monorepo/packages/client/pages/_app.tsx")
                    .canonicalize()?,
                dir.join("examples/node-monorepo/packages/client/pages/index.tsx")
                    .canonicalize()?
            ]
        );
        Ok(())
    }

    #[test]
    fn test_find_match() -> Result<()> {
        let app = App::new("./examples/node-monorepo")?;
        let re = Regex::new(r"className")?;
        let m = app.find_match(&re, "**/*.tsx").unwrap();
        assert!(m);
        Ok(())
    }

    #[test]
    fn test_strip_source_path() -> Result<()> {
        let app = App::new("./examples/node-npm")?;
        let path_to_strip = app.source.join("foo/bar.txt");
        assert_eq!(
            &app.strip_source_path(&path_to_strip).unwrap(),
            Path::new("foo/bar.txt")
        );
        Ok(())
    }

    #[test]
    fn test_strip_source_path_no_source_prefix() -> Result<()> {
        let app = App::new("./examples/node-npm")?;
        assert_eq!(
            &app.strip_source_path(Path::new("no/prefix.txt"))?,
            Path::new("no/prefix.txt")
        );
        Ok(())
    }

    #[test]
    fn test_static_asset_path() -> Result<()> {
        let app = App::new("./examples/node-npm")?;
        assert_eq!(&app.asset_path("hi.txt"), "/assets/hi.txt");
        Ok(())
    }
}
