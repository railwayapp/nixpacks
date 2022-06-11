use super::{node::NodeProvider, Provider};
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{Ok, Result};
pub struct RubyProvider {}

impl Provider for RubyProvider {
    fn name(&self) -> &str {
        "Ruby"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Gemfile"))
    }

    fn setup(&self, app: &App, env: &Environment) -> Result<Option<SetupPhase>> {
        let mut pkgs = vec![Pkg::new("gcc"), self.get_nix_ruby_package(app)];
        if app.includes_file("package.json") {
            pkgs.push(NodeProvider::get_nix_node_pkg(
                &app.read_json("package.json")?,
                env,
            )?);
        }
        let setup_phase = SetupPhase::new(pkgs);
        Ok(Some(setup_phase))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_phase = InstallPhase::new(format!(
            "gem install {} && bundle install",
            self.get_bundler_version(app)
        ));
        install_phase.add_file_dependency("Gemfile".to_string());
        if app.includes_file("Gemfile.lock") {
            install_phase.add_file_dependency("Gemfile.lock".to_string());
        }
        if app.includes_file("package.json") {
            install_phase.add_file_dependency("package.json".to_string());
            install_phase.cmd = Some(format!("npm install && {}", install_phase.cmd.unwrap()));
        }
        Ok(Some(install_phase))
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(None)
    }
}

impl RubyProvider {
    // fn detect_framework(&self, app: &App) -> String {
    //     if app.includes_file("config.ru") {
    //         "rack".to_string()
    //     } else if app.includes_file("config/environment.rb ") {
    //         "rails2".to_string()
    //     } else if app.includes_file("config/application.rb ")
    //         && app
    //             .read_file("config/application.rb ")
    //             .unwrap_or_default()
    //             .contains("Rails::Application")
    //     {
    //         "rails3".to_string()
    //     } else {
    //         "ruby".to_string()
    //     }
    // }

    fn get_nix_ruby_package(&self, app: &App) -> Pkg {
        let gemfile = app.read_file("Gemfile").unwrap_or_default();
        if gemfile.contains("ruby \"3.0.") {
            Pkg::new("ruby_3_0")
        } else if gemfile.contains("ruby \"3.1.") {
            Pkg::new("ruby_3_1")
        } else {
            Pkg::new("ruby")
        }
    }
    fn get_bundler_version(&self, app: &App) -> String {
        if app.includes_file("Gemfile.lock") {
            let gemfile_lock = app.read_file("Gemfile.lock").unwrap_or_default();
            let array_lock: Vec<&str> = gemfile_lock.split('\n').collect();
            for line in 0..array_lock.len() {
                if array_lock[line].contains("BUNDLED WITH") {
                    return format!("bundler:{}", array_lock[line + 1].trim());
                }
            }
            "bundler".to_string()
        } else {
            "bundler".to_string()
        }
    }
}
