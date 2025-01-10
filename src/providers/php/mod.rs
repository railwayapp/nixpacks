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

const LEGACY_ARCHIVE_VERSION: &str = "5148520bfab61f99fd25fb9ff7bfbb50dad3c9db";

const DEFAULT_ARCHIVE_VERSION: &str = "dbc4f15b899ac77a8d408d8e0f89fa9c0c5f2b78";

const DEFAULT_PHP_VERSION: &str = "8.3";

// (php_version, (nix_pkg_name, archive_version))
const PHP_ARCHIVE_VERSIONS: &[(&str, (&str, &str))] = &[
    ("7.4", ("php74", LEGACY_ARCHIVE_VERSION)),
    ("8.0", ("php80", LEGACY_ARCHIVE_VERSION)),
    ("8.1", ("php81", DEFAULT_ARCHIVE_VERSION)),
    ("8.2", ("php", DEFAULT_ARCHIVE_VERSION)),
    ("8.3", ("php83", DEFAULT_ARCHIVE_VERSION)),
    ("8.4", ("php84", DEFAULT_ARCHIVE_VERSION)),
];

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
        let (php_pkg, archive_version) = PhpProvider::get_php_package_and_archive(app)?;

        let mut php_extensions = PhpProvider::get_php_extensions(app).unwrap_or_default();
        php_extensions.sort_unstable();

        let mut pkgs = vec![
            Pkg::new(&format!(
                r"({}.withExtensions (pe: pe.enabled ++ [{}]))",
                &php_pkg,
                php_extensions
                    .iter()
                    .map(|ext| format!("pe.all.{ext}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )),
            Pkg::new("nginx"),
            Pkg::new("libmysqlclient"),
            Pkg::new(&format!("{}Packages.composer", &php_pkg)),
        ];
        let ext_pkgs: Vec<String> = php_extensions
            .iter()
            .map(|extension| format!("{}Extensions.{extension}", &php_pkg))
            .collect();

        pkgs.append(&mut NodeProvider::get_nix_packages(app, env)?);

        {
            let mut tmp_ext_pkgs = ext_pkgs.iter().map(|pkg| Pkg::new(pkg)).collect();
            pkgs.append(&mut tmp_ext_pkgs);
        }

        let mut phase = Phase::setup(Some(pkgs));

        phase.set_nix_archive(archive_version.to_string());
        phase.add_pkgs_libs(ext_pkgs);
        phase.add_pkgs_libs(vec!["libmysqlclient".into()]);

        Ok(phase)
    }

    fn get_install(app: &App) -> Phase {
        let mut install = Phase::install(Some(
            "mkdir -p /var/log/nginx && mkdir -p /var/cache/nginx".to_string(),
        ));
        if app.includes_file("composer.json") {
            install.add_cmd("composer install --ignore-platform-reqs".to_string());
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
        if app.includes_file("nginx.conf") {
            StartPhase::new(format!(
                "php-fpm -y {} & nginx -c /app/nginx.conf",
                app.asset_path("php-fpm.conf")
            ))
        } else if app.includes_file("nginx.template.conf") {
            StartPhase::new(format!(
                "node {} /app/nginx.template.conf /nginx.conf && (php-fpm -y {} & nginx -c /nginx.conf)",
                app.asset_path("scripts/prestart.mjs"),
                app.asset_path("php-fpm.conf"),
            ))
        } else {
            StartPhase::new(format!(
                "node {} {} /nginx.conf && (php-fpm -y {} & nginx -c /nginx.conf)",
                app.asset_path("scripts/prestart.mjs"),
                app.asset_path("nginx.template.conf"),
                app.asset_path("php-fpm.conf"),
            ))
        }
    }

    fn static_assets() -> StaticAssets {
        static_asset_list! {
            "nginx.template.conf" => include_str!("nginx.template.conf"),
            "scripts/prestart.mjs" => include_str!("scripts/prestart.mjs"),
            "php-fpm.conf" => include_str!("php-fpm.conf"),
            "scripts/util/cmd.mjs" => include_str!("scripts/util/cmd.mjs"),
            "scripts/util/nix.mjs" => include_str!("scripts/util/nix.mjs"),
            "scripts/config/template.mjs" => include_str!("scripts/config/template.mjs"),
            "scripts/util/laravel.mjs" => include_str!("scripts/util/laravel.mjs"),
            "scripts/util/logger.mjs" => include_str!("scripts/util/logger.mjs")
        }
    }

    fn environment_variables(app: &App) -> EnvironmentVariables {
        let mut vars = EnvironmentVariables::new();
        vars.insert("PORT".to_string(), "80".to_string());
        if app.includes_file("artisan") {
            vars.insert("IS_LARAVEL".to_string(), "yes".to_string());
            vars.insert(
                "NIXPACKS_PHP_ROOT_DIR".to_string(),
                "/app/public".to_string(),
            );
        }
        vars
    }

    fn get_php_package_and_archive(app: &App) -> Result<(&str, &str)> {
        let version = PhpProvider::get_php_version(app).unwrap_or(DEFAULT_PHP_VERSION.to_string());
        let (_, (pkg, archive)) = PHP_ARCHIVE_VERSIONS
            .iter()
            .find(|(php_version, _)| version == *php_version)
            .ok_or(anyhow::anyhow!("Unsupported PHP version: {}", version))?;

        Ok((pkg, archive))
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
            } else if v.contains("8.3") {
                "8.3".to_string()
            } else if v.contains("7.4") {
                "7.4".to_string()
            } else {
                eprintln!(
                    "Warning: PHP version {v} is not available, using PHP {DEFAULT_PHP_VERSION}"
                );
                DEFAULT_PHP_VERSION.to_string()
            }
        } else {
            eprintln!("Warning: No PHP version specified, using PHP {DEFAULT_PHP_VERSION}; see https://getcomposer.org/doc/04-schema.md#package-links for how to specify a PHP version.");
            DEFAULT_PHP_VERSION.to_string()
        };

        Ok(version)
    }

    fn get_php_extensions(app: &App) -> Result<Vec<String>> {
        let composer_json: ComposerJson = app.read_json("composer.json")?;
        let version = PhpProvider::get_php_version(app)?;
        let mut extensions = Vec::new();
        // ext-json is included by default in PHP >= 8.0 (and not available in Nix)
        // ext-zend-opcache is included by default in PHP >= 5.5
        let ignored_extensions = [String::from("ext-json"), String::from("ext-zend-opcache")];
        for extension in composer_json.require.keys() {
            if extension.starts_with("ext-")
                && (version == "7.4" || !ignored_extensions.contains(extension))
            {
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
