use regex::Regex;

use crate::{nixpacks::app::App, providers::node::PackageJson};

pub struct ViteSpaProvider {}

impl ViteSpaProvider {
    pub fn is_vite(app: &App) -> bool {
        app.includes_file("vite.config.js") || app.includes_file("vite.config.ts")
    }
    pub fn get_output_directory(app: &App) -> String {
        let config = app
            .read_file("vite.config.js")
            .or(app.read_file("vite.config.ts"));
        let r = Regex::new(r#"outDir:\s*['"`](.*?)['"`]"#).unwrap();
        if let Ok(config) = config {
            if let Some(c) = r.captures(&config) {
                if let Some(a) = c.get(1) {
                    return a.as_str().to_string();
                }
            }
        }
        let pkg: PackageJson = app.read_json("package.json").unwrap();
        if let Some(scripts) = pkg.scripts {
            if let Some(build) = scripts.get("build") {
                let r = Regex::new(r"vite\s+build(?:\s+-[^\s]*)*\s+(?:-o|--outDir)\s+([^-\s;]+)")
                    .unwrap();
                if let Some(c) = r.captures(build) {
                    if let Some(a) = c.get(1) {
                        return a.as_str().to_string();
                    }
                }
            }
        }
        String::from("dist")
    }
}
