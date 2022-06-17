use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};

use super::{node::NodeProvider, Provider};

const DEFAULT_PHP_VERSION: &str = "8.1";

pub struct PhpProvider;

impl Provider for PhpProvider {
    fn name(&self) -> &str {
        "php"
    }

    fn detect(&self, app: &App, _env: &Environment) -> anyhow::Result<bool> {
        Ok(app.includes_file("composer.json") || app.includes_file("index.php"))
    }

    fn setup(
        &self,
        app: &App,
        env: &Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::SetupPhase>> {
        let nodejs = NodeProvider {};

        let php_pkg = match self.get_php_package(app) {
            Ok(php_package) => php_package,
            _ => "php".to_string(),
        };
        let mut pkgs = vec![
            Pkg::new(&php_pkg),
            Pkg::new("perl"),
            Pkg::new("nginx"),
            Pkg::new(&format!("{}Packages.composer", &php_pkg)),
        ];
        if let Ok(php_extensions) = self.get_php_extensions(app) {
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
        app: &App,
        env: &Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::InstallPhase>> {
        let mut cmd = String::new();
        cmd.push_str("mkdir -p /var/log/nginx && mkdir -p /var/cache/nginx");
        let nodejs = NodeProvider {};
        if app.includes_file("composer.json") {
            cmd.push_str(" && composer install");
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
        app: &App,
        _env: &Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::BuildPhase>> {
        if let Ok(true) = NodeProvider::has_script(app, "prod") {
            return Ok(Some(BuildPhase::new(
                NodeProvider::get_package_manager(app) + " run prod",
            )));
        }
        Ok(None)
    }

    fn start(
        &self,
        app: &App,
        _env: &Environment,
    ) -> anyhow::Result<Option<crate::nixpacks::phase::StartPhase>> {
        Ok(Some(StartPhase::new(format!(
            "([ -e /app/storage ] && chmod -R ugo+w /app/storage); perl {} {} /nginx.conf && echo \"Server starting on port $PORT\" && (php-fpm -y {} & nginx -c /nginx.conf)",
            app.asset_path("transform-config.pl"),
            app.asset_path("nginx.template.conf"),
            app.asset_path("php-fpm.conf"),
        ))))
    }

    fn static_assets(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> anyhow::Result<Option<StaticAssets>> {
        Ok(Some(static_asset_list! {
            "nginx.template.conf" => include_str!("php/nginx.template.conf"),
            "transform-config.pl" => include_str!("php/transform-config.pl"),
            "php-fpm.conf" => include_str!("php/php-fpm.conf")
        }))
    }

    fn environment_variables(
        &self,
        app: &App,
        _env: &Environment,
    ) -> anyhow::Result<Option<EnvironmentVariables>> {
        let mut vars = EnvironmentVariables::new();
        if app.includes_file("artisan") {
            vars.insert("IS_LARAVEL".to_string(), "yes".to_string());
        }
        Ok(Some(vars))
    }
}

impl PhpProvider {
    fn get_php_package(&self, app: &App) -> anyhow::Result<String> {
        let version = self.get_php_version(app)?;
        Ok(format!("php{}", version.replace('.', "")))
    }
    fn get_php_version(&self, app: &App) -> anyhow::Result<String> {
        let composer_json: ComposerJson = app.read_json("composer.json")?;
        let version = composer_json.require.get("php").map(|v| v.to_string());
        Ok(match version {
            Some(v) => {
                if v.contains("8.0") {
                    "8.0".to_string()
                } else if v.contains("8.1") {
                    "8.1".to_string()
                } else if v.contains("7.4") {
                    "7.4".to_string()
                } else {
                    println!(
                        "Warning: PHP version {} is not available, using PHP {}",
                        v, DEFAULT_PHP_VERSION
                    );
                    DEFAULT_PHP_VERSION.to_string()
                }
            }
            None => {
                println!("Warning: No PHP version specified, using PHP {}; see https://getcomposer.org/doc/04-schema.md#package-links for how to specify a PHP version.", DEFAULT_PHP_VERSION);
                DEFAULT_PHP_VERSION.to_string()
            }
        })
    }
    fn get_php_extensions(&self, app: &App) -> anyhow::Result<Vec<String>> {
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
