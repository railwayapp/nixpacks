use super::node::{NodeProvider, PackageJson, DEFAULT_NODE_PKG_NAME};
use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use regex::Regex;

enum Framework {
    Rails,
    /// No framework could be found
    Vanilla,
}

pub struct RubyProvider {}

impl Provider for RubyProvider {
    fn name(&self) -> &str {
        "ruby"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Gemfile") || app.has_match("*.rb"))
    }

    fn setup(&self, app: &App, env: &Environment) -> Result<Option<SetupPhase>> {
        let framework = self.detect_framework(app);
        let mut packages = vec![Pkg::new("ruby")];
        let needs_java = app.find_match(&Regex::new(r"jruby")?, "Gemfile.lock")?;
        if needs_java {
            packages.push(Pkg::new("java"));
        }
        match framework {
            Framework::Rails => {
                packages.push(Pkg::new("postgresql"));
                if app.includes_file("package.json") {
                    let package_json: PackageJson = app.read_json("package.json")?;
                    let node_pkg = NodeProvider::get_nix_node_pkg(&package_json, env)?;
                    if NodeProvider::get_package_manager(app)? == "pnpm" {
                        let mut pnpm_pkg = Pkg::new("nodePackages.pnpm");
                        // Only override the node package if not the default one
                        if node_pkg.name != *DEFAULT_NODE_PKG_NAME {
                            pnpm_pkg = pnpm_pkg.set_override("nodejs", node_pkg.name.as_str());
                        }
                        packages.push(pnpm_pkg);
                    } else if NodeProvider::get_package_manager(app)? == "yarn" {
                        let mut yarn_pkg = Pkg::new("yarn");
                        // Only override the node package if not the default one
                        if node_pkg.name != *DEFAULT_NODE_PKG_NAME {
                            yarn_pkg = yarn_pkg.set_override("nodejs", node_pkg.name.as_str());
                        }
                        packages.push(yarn_pkg)
                    }
                }
                Ok(Some(SetupPhase::new(packages)))
            }
            Framework::Vanilla => Ok(Some(SetupPhase::new(packages))),
        }
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_cmd = Vec::<&str>::new();
        if NodeProvider::get_package_manager(app)? == "pnpm" {
            install_cmd.push("pnpm i --frozen-lockfile");
        } else if NodeProvider::get_package_manager(app)? == "yarn" {
            if app.includes_file(".yarnrc.yml") {
                install_cmd
                    .push("yarn set version berry && yarn install --immutable --check-cache");
            } else {
                install_cmd.push("yarn install --frozen-lockfile --production=false");
            }
        } else if app.includes_file("package-lock.json") {
            install_cmd.push("npm ci");
        }
        if app.includes_file("Gemfile") {
            install_cmd.push("bundle install --frozen");
        }

        if install_cmd.is_empty() {
            Ok(None)
        } else {
            Ok(Some(InstallPhase::new(install_cmd.join(" && "))))
        }
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(None)
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        if app.includes_file("main.rb") {
            return Ok(Some(StartPhase::new(String::from(
                "bundle exec ruby main.rb",
            ))));
        }
        Ok(None)
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(None)
    }
}

impl RubyProvider {
    fn detect_framework(&self, app: &App) -> Framework {
        if app.includes_file("Rakefile") {
            Framework::Rails
        } else {
            Framework::Vanilla
        }
    }
}
