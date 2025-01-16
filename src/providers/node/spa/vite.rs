use regex::Regex;

use crate::{nixpacks::app::App, providers::node::PackageJson};

pub struct ViteSpaProvider {}

impl ViteSpaProvider {
    pub fn is_vite(app: &App) -> bool {
        let package_json: PackageJson = app.read_json("package.json").unwrap_or_default();
        package_json.has_dependency("vite")
            || app.includes_file("vite.config.js")
            || app.includes_file("vite.config.ts")
            || {
                let pkg: PackageJson = app.read_json("package.json").unwrap_or_default();
                if let Some(scripts) = pkg.scripts {
                    if let Some(build) = scripts.get("build") {
                        build.to_lowercase().contains("vite build")
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
    }

    pub fn caddy_allowlist(app: &App) -> bool {
        let pkg: PackageJson = app.read_json("package.json").unwrap();
        pkg.has_dependency("react")
            || pkg.has_dependency("vue")
            || (pkg.has_dependency("svelte") && !pkg.has_dependency("@sveltejs/kit"))
            || pkg.has_dependency("preact")
            || pkg.has_dependency("lit")
            || pkg.has_dependency("solid-js")
            || pkg.has_dependency("@builder.io/qwik")
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
                let r =
                    Regex::new(r"vite\s+build(?:\s+-[^\s]*)*\s+(?:--outDir)\s+([^-\s;]+)").unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vite_no_special() {
        // no special config
        let app = crate::nixpacks::app::App::new("examples/node-vite-react-ts").unwrap();
        assert_eq!(ViteSpaProvider::get_output_directory(&app), "dist");
    }

    #[test]
    fn test_vite_outdir_in_config() {
        // outDir specified in vite.config.js
        let app = crate::nixpacks::app::App::new("examples/node-vite-svelte-ts").unwrap();
        assert_eq!(ViteSpaProvider::get_output_directory(&app), "build");
    }

    #[test]
    fn test_vite_outdir_in_build_cmd() {
        // outDir specified in vite.config.js
        let app = crate::nixpacks::app::App::new("examples/node-vite-solid-ts").unwrap();
        assert_eq!(ViteSpaProvider::get_output_directory(&app), "out");
    }

    #[test]
    fn test_not_match() {
        // should not match
        let app = crate::nixpacks::app::App::new("examples/node-bun-web-server").unwrap();
        assert!(!ViteSpaProvider::is_vite(&app));
    }

    #[test]
    fn test_not_match_2() {
        // should not match
        let app = crate::nixpacks::app::App::new("examples/node").unwrap();
        assert!(!ViteSpaProvider::is_vite(&app));
    }
}
