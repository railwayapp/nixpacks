pub struct CobolProvider {}

impl Provider for CobolProvider {
    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("index.cbl"))
    }
}
