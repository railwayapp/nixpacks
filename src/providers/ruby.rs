use super::{node::NodeProvider, Provider};
use crate::nixpacks::{
    app::App,
    environment::Environment,
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
            pkgs = NodeProvider::get_nix_packages(app, env)?
        }
        let mut setup_phase = SetupPhase::new(pkgs);
        setup_phase.add_apt_pkgs(vec!["procps".to_string()]);
        setup_phase.add_cmd(
            "curl -sSL https://get.rvm.io | bash -s stable && source /etc/profile.d/rvm.sh"
                .to_string(),
        );
        setup_phase.add_cmd("rvm install ".to_string() + &self.get_ruby_version(app));
        setup_phase.add_cmd("gem install ".to_string() + &self.get_bundler_version(app));
        Ok(Some(setup_phase))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_phase = InstallPhase::new("bundle install".to_string());
        install_phase.add_file_dependency("Gemfile".to_string());
        add_file_if_included(app, &mut install_phase, "Gemfile.lock");
        if app.includes_file("package.json") {
            install_phase.add_file_dependency("package.json".to_string());
            install_phase.cmd = Some(formatdoc!(
                "{} && {}",
                NodeProvider::get_install_command(app),
                install_phase.cmd.unwrap()
            ));
            add_file_if_included(app, &mut install_phase, "package-lock.json");
            add_file_if_included(app, &mut install_phase, "yarn.lock");
            add_file_if_included(app, &mut install_phase, "pnpm-lock.yaml");
        }
        Ok(Some(install_phase))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(Some(StartPhase::new(self.get_start_command(app))))
    }
}

impl RubyProvider {
    fn get_start_command(&self, app: &App) -> String {
        if app.includes_file("config.ru") {
            "bundle exec rackup config.ru -p ${PORT:-3000}".to_string()
        } else if app.includes_file("config/application.rb")
            && app
                .read_file("config/application.rb")
                .unwrap_or_default()
                .contains("Rails::Application")
        {
            if app.includes_file("rails") {
                "bundle exec rails server -b 0.0.0.0 -p ${PORT:-3000}".to_string()
            } else {
                "bundle exec bin/rails server -b 0.0.0.0 -p ${PORT:-3000} -e $RAILS_ENV".to_string()
            }
        } else if app.includes_file("config/environment.rb") && app.includes_directory("script") {
            "bundle exec ruby script/server -p ${PORT:-3000}".to_string()
        } else {
            "bundle exec rake".to_string()
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
                if array_lock[line].contains("BUNDLED WITH") && line + 1 < array_lock.len() {
                    return format!("bundler:{}", array_lock[line + 1].trim());
                }
            }
            "bundler".to_string()
        } else {
            "bundler".to_string()
        }
    }
}

fn add_file_if_included(app: &App, phase: &mut InstallPhase, file: &str) {
    if app.includes_file(file) {
        phase.add_file_dependency(file.to_string());
    }
}
