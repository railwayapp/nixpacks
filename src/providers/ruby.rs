use super::{node::NodeProvider, Provider};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{bail, Ok, Result};
use regex::Regex;

pub struct RubyProvider {}

const BUNDLE_CACHE_DIR: &str = "/root/.bundle/cache";

impl Provider for RubyProvider {
    fn name(&self) -> &str {
        "Ruby"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Gemfile"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = self.get_setup(app, env)?;
        let install = self.get_install(app)?;
        let build = self.get_build(app)?;
        let start = self.get_start(app)?;

        let mut plan = BuildPlan::new(
            vec![setup, install, build]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            start,
        );

        let node = NodeProvider {};
        if node.detect(app, env)? {
            let node_build_plan = node.get_build_plan(app, env)?;
            if let Some(node_build_plan) = node_build_plan {
                // Include the install phase from the node provider
                let phase_name =
                    plan.add_phases_from_another_plan(&node_build_plan, node.name(), "install");
                plan.add_dependency_between_phases("build", phase_name.as_str());
            }
        }

        plan.add_variables(self.get_environment_variables(app)?);

        Ok(Some(plan))
    }
}

impl RubyProvider {
    fn get_setup(&self, app: &App, _env: &Environment) -> Result<Option<Phase>> {
        let mut setup = Phase::setup(None);
        setup.add_apt_pkgs(vec!["procps".to_string()]);

        if self.uses_postgres(app)? {
            setup.add_apt_pkgs(vec!["libpq-dev".to_string()]);
        }

        if self.uses_mysql(app)? {
            setup.add_apt_pkgs(vec!["default-libmysqlclient-dev".to_string()]);
        }

        setup.add_cmd(
            "curl -sSL https://get.rvm.io | bash -s stable && . /etc/profile.d/rvm.sh".to_string(),
        );

        setup.add_cmd(format!("rvm install {}", self.get_ruby_version(app)?));
        setup.add_cmd(format!("rvm --default use {}", self.get_ruby_version(app)?));
        setup.add_cmd(format!("gem install {}", self.get_bundler_version(app)));
        setup.add_cmd("echo 'source /usr/local/rvm/scripts/rvm' >> /root/.profile".to_string());

        Ok(Some(setup))
    }

    fn get_install(&self, app: &App) -> Result<Option<Phase>> {
        let mut install = Phase::install(None);
        install.add_cache_directory(BUNDLE_CACHE_DIR.to_string());

        install.add_cmd("bundle install".to_string());

        // Ensure that the ruby executable is in the PATH
        let ruby_version = self.get_ruby_version(app)?;
        install.add_path(format!("/usr/local/rvm/rubies/{}/bin", ruby_version));
        install.add_path(format!("/usr/local/rvm/gems/{}/bin", ruby_version));
        install.add_path(format!("/usr/local/rvm/gems/{}@global/bin", ruby_version));

        Ok(Some(install))
    }

    fn get_build(&self, app: &App) -> Result<Option<Phase>> {
        if self.is_rails_app(app) {
            Ok(Some(Phase::build(Some(
                "bundle exec rake assets:precompile".to_string(),
            ))))
        } else {
            Ok(None)
        }
    }

    fn get_start(&self, app: &App) -> Result<Option<StartPhase>> {
        if let Some(start_cmd) = self.get_start_command(app) {
            Ok(Some(StartPhase::new(start_cmd)))
        } else {
            Ok(None)
        }
    }

    fn get_environment_variables(&self, app: &App) -> Result<EnvironmentVariables> {
        let ruby_version = self.get_ruby_version(app)?;
        Ok(EnvironmentVariables::from([
            ("BUNDLE_GEMFILE".to_string(), "/app/Gemfile".to_string()),
            (
                "GEM_PATH".to_string(),
                format!(
                    "/usr/local/rvm/gems/{ruby_version}:/usr/local/rvm/gems/{ruby_version}@global",
                    ruby_version = ruby_version
                ),
            ),
            (
                "GEM_HOME".to_string(),
                format!("/usr/local/rvm/gems/{}", ruby_version),
            ),
        ]))
    }

    fn get_start_command(&self, app: &App) -> Option<String> {
        if self.is_rails_app(app) {
            if app.includes_file("rails") {
                Some("bundle exec rails server -b 0.0.0.0 -p ${PORT:-3000}".to_string())
            } else {
                Some(
                    "bundle exec bin/rails server -b 0.0.0.0 -p ${PORT:-3000} -e $RAILS_ENV"
                        .to_string(),
                )
            }
        } else if app.includes_file("config/environment.rb") && app.includes_directory("script") {
            Some("bundle exec ruby script/server -p ${PORT:-3000}".to_string())
        } else if app.includes_file("config.ru") {
            Some("bundle exec rackup config.ru -p ${PORT:-3000}".to_string())
        } else if app.includes_file("Rakefile") {
            Some("bundle exec rake".to_string())
        } else {
            None
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

    fn uses_postgres(&self, app: &App) -> Result<bool> {
        if app.includes_file("Gemfile") {
            let gemfile = app.read_file("Gemfile").unwrap_or_default();
            return Ok(gemfile.contains("pg"));
        }
        Ok(false)
    }
    fn uses_mysql(&self, app: &App) -> Result<bool> {
        if app.includes_file("Gemfile") {
            let gemfile = app.read_file("Gemfile").unwrap_or_default();
            return Ok(gemfile.contains("mysql"));
        }
        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gemfile_lock_version() -> Result<()> {
        assert_eq!(
            RubyProvider::get_ruby_version(&RubyProvider {}, &App::new("./examples/ruby")?)?,
            "ruby-3.1.2"
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

    #[test]
    fn test_version_file() -> Result<()> {
        assert_eq!(
            RubyProvider::get_ruby_version(
                &RubyProvider {},
                &App::new("./examples/ruby-rails-postgres")?
            )?,
            "3.1.2"
        );

        Ok(())
    }
}
