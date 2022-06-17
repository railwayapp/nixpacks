use super::{node::NodeProvider, Provider};
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{Ok, Result};
use indoc::formatdoc;
pub struct RubyProvider {}

impl Provider for RubyProvider {
    fn name(&self) -> &str {
        "Ruby"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Gemfile"))
    }

    fn setup(&self, app: &App, env: &Environment) -> Result<Option<SetupPhase>> {
        let mut pkgs = vec![];
        if app.includes_file("package.json") {
            pkgs.push(NodeProvider::get_nix_node_pkg(
                &app.read_json("package.json")?,
                env,
            )?);
            pkgs.push(Pkg::new("yarn"));
        }
        let mut setup_phase = SetupPhase::new(pkgs);
        setup_phase.add_apt_pkgs(vec!["procps".to_string()]);
        Ok(Some(setup_phase))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let rvm_install_cmd =
            "curl -sSL https://get.rvm.io | bash -s stable && source /etc/profile.d/rvm.sh"
                .to_string();
        let install_cmd = formatdoc!(
            "{}
            RUN rvm install {} 
            RUN gem install {} 
            RUN bundle install",
            rvm_install_cmd,
            self.get_ruby_version(app),
            self.get_bundler_version(app)
        );
        let mut install_phase = InstallPhase::new(install_cmd);
        install_phase.add_file_dependency("Gemfile".to_string());
        if app.includes_file("Gemfile.lock") {
            install_phase.add_file_dependency("Gemfile.lock".to_string());
        }
        if app.includes_file("package.json") {
            install_phase.add_file_dependency("package.json".to_string());
            install_phase.cmd = Some(formatdoc!(
                "
                yarn install
                RUN {}",
                install_phase.cmd.unwrap()
            ));
        }
        Ok(Some(install_phase))
    }

    fn start(&self, _app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(None)
    }
}

impl RubyProvider {
    fn _detect_framework(&self, app: &App) -> String {
        if app.includes_file("config.ru") {
            "rack".to_string()
        } else if app.includes_file("config/environment.rb ") {
            "rails2".to_string()
        } else if app.includes_file("config/application.rb ")
            && app
                .read_file("config/application.rb ")
                .unwrap_or_default()
                .contains("Rails::Application")
        {
            "rails3".to_string()
        } else {
            "ruby".to_string()
        }
    }

    fn get_ruby_version(&self, app: &App) -> String {
        app.read_file(".ruby-version").unwrap_or_default()
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
