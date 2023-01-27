use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::nixpacks::{
    app::{App, StaticAssets},
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};

use super::{node::NodeProvider, Provider};
use anyhow::Result;

const DEFAULT_PHP_VERSION: &str = "8.2";

pub struct PhpProvider;

impl Provider for PhpProvider {
    fn name(&self) -> &str {
        "php"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("composer.json") || app.includes_file("index.php"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = PhpProvider::get_setup(app, env)?;
        let install = PhpProvider::get_install(app);
        let build = PhpProvider::get_build(app);
        let start = PhpProvider::get_start(app);

        let mut plan = BuildPlan::new(
            &vec![Some(setup), Some(install), build]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            Some(start),
        );

        plan.add_static_assets(PhpProvider::static_assets());
        plan.add_variables(PhpProvider::environment_variables(app));

        Ok(Some(plan))
    }
}

impl PhpProvider {
    fn get_setup(app: &App, env: &Environment) -> Result<Phase> {
        let php_pkg = match PhpProvider::get_php_package(app) {
            Ok(php_package) => php_package,
            _ => "php".to_string(),
        };
        let mut pkgs = vec![
            Pkg::new(&php_pkg),
            Pkg::new("perl"),
            Pkg::new("nginx"),
            Pkg::new(&format!("{}Packages.composer", &php_pkg)),
        ];
        if let Ok(php_extensions) = PhpProvider::get_php_extensions(app) {
            for extension in php_extensions {
                pkgs.push(Pkg::new(&format!("{}Extensions.{extension}", &php_pkg)));
            }
        }

        if app.includes_file("package.json") {
            pkgs.append(&mut NodeProvider::get_nix_packages(app, env)?);
        }

        Ok(Phase::setup(Some(pkgs)))
    }

    fn get_install(app: &App) -> Phase {
        let mut install = Phase::install(Some(
            "mkdir -p /var/log/nginx && mkdir -p /var/cache/nginx".to_string(),
        ));
        if app.includes_file("composer.json") {
            install.add_cmd("composer install".to_string());
        };
        if app.includes_file("package.json") {
            if let Some(install_cmd) = NodeProvider::get_install_command(app) {
                install.add_cmd(install_cmd);
            }
        }

        install
    }

    fn get_build(app: &App) -> Option<Phase> {
        if let Ok(true) = NodeProvider::has_script(app, "prod") {
            return Some(Phase::build(Some(
                NodeProvider::get_package_manager(app) + " run prod",
            )));
        } else if let Ok(true) = NodeProvider::has_script(app, "build") {
            return Some(Phase::build(Some(
                NodeProvider::get_package_manager(app) + " run build",
            )));
        }

        None
    }

    fn get_start(app: &App) -> StartPhase {
        StartPhase::new(format!(
            "([ -e /app/storage ] && chmod -R ugo+w /app/storage); perl {} {} /nginx.conf && echo \"Server starting on port $PORT\" && (php-fpm -y {} & nginx -c /nginx.conf)",
            app.asset_path("transform-config.pl"),
            app.asset_path("nginx.template.conf"),
            app.asset_path("php-fpm.conf"),
        ))
    }

    fn static_assets() -> StaticAssets {
        static_asset_list! {
            "nginx.template.conf" => include_str!("nginx.template.conf"),
            "transform-config.pl" => include_str!("transform-config.pl"),
            "php-fpm.conf" => include_str!("php-fpm.conf")
        }
    }

    fn environment_variables(app: &App) -> EnvironmentVariables {
        let mut vars = EnvironmentVariables::new();
        vars.insert("PORT".to_string(), "80".to_string());
        if app.includes_file("artisan") {
            vars.insert("IS_LARAVEL".to_string(), "yes".to_string());
        }
        vars
    }

    fn get_php_package(app: &App) -> Result<String> {
        let version = PhpProvider::get_php_version(app)?;
        Ok(format!("php{}", version.replace('.', "")))
    }

    fn get_php_version(app: &App) -> Result<String> {
        let composer_json: ComposerJson = app.read_json("composer.json")?;
        let version = composer_json.require.get("php").cloned();

        let version = if let Some(v) = version {
            if v.contains("8.0") {
                "8.0".to_string()
            } else if v.contains("8.1") {
                "8.1".to_string()
            } else if v.contains("8.2") {
                "8.2".to_string()
            } else if v.contains("7.4") {
                "7.4".to_string()
            } else {
                println!(
                    "Warning: PHP version {v} is not available, using PHP {DEFAULT_PHP_VERSION}"
                );
                DEFAULT_PHP_VERSION.to_string()
            }
        } else {
            println!("Warning: No PHP version specified, using PHP {DEFAULT_PHP_VERSION}; see https://getcomposer.org/doc/04-schema.md#package-links for how to specify a PHP version.");
            DEFAULT_PHP_VERSION.to_string()
        };

        Ok(version)
    }

    fn get_php_extensions(app: &App) -> Result<Vec<String>> {
        let composer_json: ComposerJson = app.read_json("composer.json")?;
        let mut extensions = Vec::new();
        for extension in composer_json.require.keys() {
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
