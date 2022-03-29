use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf};

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
        for path in &self.paths {
            if path.file_name().unwrap() == name {
                return true;
            }
        }

        false
    }

    pub fn read_file(&self, name: &str) -> Result<String> {
        let name = self.source.join(name);
        let contents = fs::read_to_string(name)?;
        Ok(contents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creates_app() -> Result<()> {
        let app = App::new("./tests/fixtures/npm")?;
        assert_eq!(app.paths.len(), 4);
        Ok(())
    }

    #[test]
    fn test_read_file() -> Result<()> {
        let app = App::new("./tests/fixtures/npm")?;
        assert_eq!(
            app.read_file("index.ts")?.trim_end(),
            "console.log(\"Hello from NPM\");"
        );
        Ok(())
    }
}
