use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::nixpacks::{
    nix::Pkg,
    phase::{InstallPhase, SetupPhase},
};

use super::{node::NodeProvider, Provider};

pub struct PhpProvider;

impl Provider for PhpProvider {
    fn name(&self) -> &str {
        "php"
    }

    fn detect(
        &self,
        app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<bool> {
        Ok(app.includes_file("composer.json") || app.includes_file("index.php"))
    }

    fn setup(
        &self,
        app: &crate::nixpacks::app::App,
        env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::SetupPhase>> {
        let nodejs = NodeProvider {};

        let php_pkg = match self.get_php_package(&app) {
            Ok(Some(php_package)) => php_package,
            _ => "php".to_string(),
        };
        let mut pkgs = vec![
            Pkg::new(&php_pkg),
            Pkg::new("nginx"),
            Pkg::new(&format!("{}Packages.composer", &php_pkg)),
        ];
        if let Ok(php_extensions) = self.get_php_extensions(&app) {
            for extension in php_extensions {
                pkgs.push(Pkg::new(&format!("{}Extensions.{}", &php_pkg, extension)));
            }
        }

        if let Ok(true) = nodejs.detect(app, env) {
            if let Ok(Some(mut node_setup)) = nodejs.setup(app, env) {
                pkgs.append(&mut node_setup.pkgs);
            }
        }

        Ok(Some(SetupPhase::new(pkgs)))
    }

    fn install(
        &self,
        app: &crate::nixpacks::app::App,
        env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::InstallPhase>> {
        let mut cmd = String::new();
        let nodejs = NodeProvider {};
        if app.includes_file("composer.json") {
            cmd.push_str("composer install");
        };

        if nodejs.detect(app, env)? {
            if !cmd.is_empty() {
                cmd.push_str(" && ");
            }
            cmd.push_str(
                nodejs
                    .install(app, env)?
                    .as_ref()
                    .unwrap()
                    .cmd
                    .as_ref()
                    .unwrap()
                    .as_str(),
            );
        }
        Ok(Some(InstallPhase::new(cmd)))
    }

    fn build(
        &self,
        app: &crate::nixpacks::app::App,
        env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::BuildPhase>> {
        Ok(None)
    }

    fn start(
        &self,
        _app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::StartPhase>> {
        Ok(None)
    }

    fn environment_variables(
        &self,
        _app: &crate::nixpacks::app::App,
        _env: &crate::nixpacks::environment::Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::environment::EnvironmentVariables>> {
        Ok(None)
    }
}

impl PhpProvider {
    fn get_php_package(&self, app: &crate::nixpacks::app::App) -> anyhow::Result<Option<String>> {
        let composer_json: ComposerJson = app.read_json("composer.json")?;
        let version = composer_json.require.get("php").map(|v| v.to_string());
        Ok(match version {
            Some(v) => {
                if v.contains("8.0") {
                    Some("php80".to_string())
                } else if v.contains("8.1") {
                    Some("php81".to_string())
                } else if v.contains("7.4") {
                    Some("php74".to_string())
                } else {
                    println!("Warning: PHP version {} is not available, using PHP 8.1", v);
                    Some("php81".to_string())
                }
            }
            None => {
                println!("Warning: No PHP version specified, using PHP 8.1; see https://getcomposer.org/doc/04-schema.md#package-links for how to specify a PHP version.");
                Some("php81".to_string())
            }
        })
    }
    fn get_php_extensions(&self, app: &crate::nixpacks::app::App) -> anyhow::Result<Vec<String>> {
        let composer_json: ComposerJson = app.read_json("composer.json")?;
        let mut extensions = Vec::new();
        for (extension, _) in composer_json.require.iter() {
            if extension.starts_with("ext-") {
                extensions.push(
                    extension
                        .strip_prefix("ext-")
                        .unwrap_or(extension)
                        .to_string(),
                );
            }
        }
        Ok(extensions)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ComposerJson {
    require: HashMap<String, String>,
}
