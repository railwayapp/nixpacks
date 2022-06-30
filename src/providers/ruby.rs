use super::{node::NodeProvider, Provider};
use crate::nixpacks::{
    app::App,
    environment::Environment,
    phase::{InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{bail, Ok, Result};
use regex::Regex;

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
            "curl -sSL https://get.rvm.io | bash -s stable && . /etc/profile.d/rvm.sh".to_string(),
        );
        setup_phase.add_cmd("rvm install ".to_string() + &self.get_ruby_version(app).unwrap());
        setup_phase.add_cmd("gem install ".to_string() + &self.get_bundler_version(app));
        Ok(Some(setup_phase))
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        let mut install_phase = InstallPhase::new("bundle install".to_string());
        install_phase.add_file_dependency("Gemfile*".to_string());
        if app.includes_file("package.json") {
            install_phase.add_file_dependency("package.json".to_string());
            install_phase
                .cmds
                .clone()
                .unwrap_or_default()
                .insert(0, NodeProvider::get_install_command(app));

            for file in ["package.json", "package-lock.json"].iter() {
                if app.includes_file(file) {
                    install_phase.add_file_dependency(file.to_string());
                }
            }
        }
        Ok(Some(install_phase))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        Ok(Some(StartPhase::new(self.get_start_command(app))))
    }
}

impl RubyProvider {
    fn get_start_command(&self, app: &App) -> String {
        if self.is_rails_app(app) {
            if app.includes_file("rails") {
                "bundle exec rails server -b 0.0.0.0 -p ${PORT:-3000}".to_string()
            } else {
                "bundle exec bin/rails server -b 0.0.0.0 -p ${PORT:-3000} -e $RAILS_ENV".to_string()
            }
        } else if app.includes_file("config/environment.rb") && app.includes_directory("script") {
            "bundle exec ruby script/server -p ${PORT:-3000}".to_string()
        } else if app.includes_file("config.ru") {
            "bundle exec rackup config.ru -p ${PORT:-3000}".to_string()
        } else {
            "bundle exec rake".to_string()
        }
    }

    fn get_ruby_version(&self, app: &App) -> Result<String> {
        if app.includes_file(".ruby-version") {
            return Ok(app.read_file(".ruby-version")?.trim().to_string());
        }
        let re_gemfile = Regex::new(r#"ruby (?:'|")(.*)(?:'|")[^>]"#).unwrap();
        let gemfile = app.read_file("Gemfile").unwrap_or_default();
        if let Some(value) = re_gemfile.captures(&gemfile) {
            return Ok(format!("ruby-{}", value.get(1).unwrap().as_str()));
        }
        let re_gemfile_lock =
            Regex::new(r#"ruby ((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*))[^>]"#).unwrap();
        let gemfile_lock = app.read_file("Gemfile.lock").unwrap_or_default();
        if let Some(value) = re_gemfile_lock.captures(&gemfile_lock) {
            return Ok(format!("ruby-{}", value.get(1).unwrap().as_str()));
        }
        bail!("Please specify ruby's version in .ruby-version file")
    }

    // Loop through Gemfile.lock and find bundler's version (Line below BUNDLED WITH)
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

    fn is_rails_app(&self, app: &App) -> bool {
        app.includes_file("config/application.rb")
            && app
                .read_file("config/application.rb")
                .unwrap_or_default()
                .contains("Rails::Application")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gemfile_version() -> Result<()> {
        assert_eq!(
            RubyProvider::get_ruby_version(
                &RubyProvider {},
                &App::new("./examples/ruby-gemfile")?
            )?,
            "ruby-2.7.2"
        );

        Ok(())
    }

    #[test]
    fn test_gemfile_lock_version() -> Result<()> {
        assert_eq!(
            RubyProvider::get_ruby_version(
                &RubyProvider {},
                &App::new("./examples/ruby-gemfile-lock")?
            )?,
            "ruby-2.7.2"
        );

        Ok(())
    }

    #[test]
    fn test_no_version() -> Result<()> {
        assert!(RubyProvider::get_ruby_version(
            &RubyProvider {},
            &App::new("./examples/ruby-no-version")?,
        )
        .is_err());
        Ok(())
    }
}
