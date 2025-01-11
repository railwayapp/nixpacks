use super::{node::NodeProvider, Provider};
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{bail, Ok, Result};
use regex::Regex;

struct RubyVersion {
    major: u8,
    minor: u8,
}

impl RubyVersion {
    fn parse(version: &str) -> Option<Self> {
        let mut split = version.split('.');
        let major = split.next()?.parse().ok()?;
        let minor = split.next()?.parse().ok()?;
        Some(Self { major, minor })
    }
}

pub struct RubyProvider {}

const BUNDLE_CACHE_DIR: &str = "/root/.bundle/cache";

impl Provider for RubyProvider {
    fn name(&self) -> &'static str {
        "ruby"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("Gemfile"))
    }

    fn get_build_plan(&self, app: &App, env: &Environment) -> Result<Option<BuildPlan>> {
        let setup = self.get_setup(app, env)?;
        let install = self.get_install(app, env)?;
        let build = self.get_build(app)?;
        let start = self.get_start(app)?;

        let mut plan = BuildPlan::new(
            &vec![setup, install, build]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            start,
        );

        let node = NodeProvider::default();
        if node.detect(app, env)? || self.uses_gem_dep(app, "execjs") {
            let node_build_plan = node.get_build_plan(app, env)?;
            if let Some(node_build_plan) = node_build_plan {
                // Include the install phase from the node provider
                let root_phase_name =
                    plan.add_phases_from_another_plan(&node_build_plan, node.name(), "install");
                plan.add_dependency_between_phases("build", root_phase_name.as_str());
            }
        }

        plan.add_variables(self.get_environment_variables(app, env)?);

        Ok(Some(plan))
    }
}

impl RubyProvider {
    fn get_setup(&self, app: &App, env: &Environment) -> Result<Option<Phase>> {
        let mut setup = Phase::setup(None);
        setup.add_apt_pkgs(vec!["procps".to_string()]);

        // Don't re-install ruby if the code has changed
        setup.only_include_files = Some(Vec::new());

        if self.uses_postgres(app)? {
            setup.add_apt_pkgs(vec!["libpq-dev".to_string()]);
        }

        if self.uses_mysql(app)? {
            setup.add_apt_pkgs(vec!["default-libmysqlclient-dev".to_string()]);
        }

        if self.uses_gem_dep(app, "magick") {
            setup.add_apt_pkgs(vec![String::from("libmagickwand-dev")]);
            setup.add_nix_pkgs(&[Pkg::new("imagemagick")]);
        }

        if self.uses_gem_dep(app, "vips") {
            setup.add_apt_pkgs(vec![String::from("libvips-dev")]);
        }

        if self.uses_gem_dep(app, "charlock_holmes") {
            setup.add_apt_pkgs(vec![String::from("libicu-dev")]);
        }

        let ruby_version = self.get_ruby_version(app, env)?;
        let ruby_version = ruby_version.trim_start_matches("ruby-");

        if let Some(ruby_version) = RubyVersion::parse(ruby_version) {
            // YJIT in Ruby 3.1+ requires rustc to install
            if ruby_version.major >= 3 && ruby_version.minor >= 1 {
                setup.add_nix_pkgs(&[Pkg::new("rustc")]);
            }
        }

        // Packages necessary for rbenv
        // https://github.com/rbenv/ruby-build/wiki#ubuntudebianmint
        setup.add_apt_pkgs(
            vec![
                "git",
                "curl",
                "autoconf",
                "bison",
                "build-essential",
                "libssl-dev",
                "libyaml-dev",
                "libreadline6-dev",
                "zlib1g-dev",
                "libncurses5-dev",
                "libffi-dev",
                "libgdbm6",
                "libgdbm-dev",
                "libdb-dev",
            ]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        );

        let bundler_version = self.get_bundler_version(app);

        setup.add_cmd(format!(
            "curl -fsSL https://github.com/rbenv/rbenv-installer/raw/HEAD/bin/rbenv-installer | bash -s stable \
            && printf '\\neval \"$(~/.rbenv/bin/rbenv init -)\"' >> /root/.profile \
            && . /root/.profile \
            && rbenv install {ruby_version} \
            && rbenv global {ruby_version} \
            && gem install {bundler_version}"
        ));

        setup.add_path("$HOME/.rbenv/bin".to_string());

        Ok(Some(setup))
    }

    fn get_install(&self, app: &App, env: &Environment) -> Result<Option<Phase>> {
        let mut install = Phase::install(None);
        install.add_cache_directory(BUNDLE_CACHE_DIR.to_string());

        if !self.uses_gem_dep(app, "local") {
            // Only run install if Gemfile or Gemfile.lock has changed
            install.only_include_files =
                Some(vec!["Gemfile".to_string(), "Gemfile.lock".to_string()]);
        }

        install.add_cmd("bundle install".to_string());

        if self.uses_gem_dep(app, "bootsnap") {
            install.add_cmd("bundle exec bootsnap precompile --gemfile");
        }

        // Ensure that the ruby executable is in the PATH
        let ruby_version = self.get_ruby_version(app, env)?;
        install.add_path(format!("/usr/local/rvm/rubies/{ruby_version}/bin"));
        install.add_path(format!("/usr/local/rvm/gems/{ruby_version}/bin"));
        install.add_path(format!("/usr/local/rvm/gems/{ruby_version}@global/bin"));

        Ok(Some(install))
    }

    fn get_build(&self, app: &App) -> Result<Option<Phase>> {
        let mut build = Phase::build(None);

        // Only compile assets if a Rails app have an asset pipeline gem
        // installed (e.g. sprockets, propshaft). Rails API-only apps [0]
        // do not come with the asset pipelines because they have no assets.
        // [0] https://guides.rubyonrails.org/api_app.html
        if self.is_rails_app(app) && self.uses_asset_pipeline(app)? {
            build.add_cmd("bundle exec rake assets:precompile".to_string());
        }

        if self.is_rails_app(app) && self.uses_gem_dep(app, "bootsnap") {
            build.add_cmd("bundle exec bootsnap precompile app/ lib/");
        }

        Ok(Some(build))
    }

    fn get_start(&self, app: &App) -> Result<Option<StartPhase>> {
        if let Some(start_cmd) = self.get_start_command(app) {
            Ok(Some(StartPhase::new(start_cmd)))
        } else {
            Ok(None)
        }
    }

    fn get_environment_variables(
        &self,
        app: &App,
        env: &Environment,
    ) -> Result<EnvironmentVariables> {
        let ruby_version = self.get_ruby_version(app, env)?;
        let mut env_vars = EnvironmentVariables::from([
            ("BUNDLE_GEMFILE".to_string(), "/app/Gemfile".to_string()),
            (
                "GEM_PATH".to_string(),
                format!(
                    "/usr/local/rvm/gems/{ruby_version}:/usr/local/rvm/gems/{ruby_version}@global"
                ),
            ),
            (
                "GEM_HOME".to_string(),
                format!("/usr/local/rvm/gems/{ruby_version}"),
            ),
            ("MALLOC_ARENA_MAX".to_string(), "2".to_string()),
        ]);

        if self.is_rails_app(app) {
            env_vars.insert("RAILS_LOG_TO_STDOUT".to_string(), "enabled".to_string());
            env_vars.insert("RAILS_SERVE_STATIC_FILES".to_string(), "1".to_string());
        }

        Ok(env_vars)
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

    fn get_ruby_version(&self, app: &App, env: &Environment) -> Result<String> {
        if let Some(version) = env.get_config_variable("RUBY_VERSION") {
            return Ok(version);
        }
        if app.includes_file(".ruby-version") {
            return Ok(app.read_file(".ruby-version")?.trim().to_string());
        }
        let re_gemfile = Regex::new(r#"ruby (?:'|")(.*)(?:'|")[^>]"#).unwrap();
        let gemfile = app.read_file("Gemfile").unwrap_or_default();
        if let Some(value) = re_gemfile.captures(&gemfile) {
            return Ok(format!("ruby-{}", value.get(1).unwrap().as_str()));
        }
        let re_gemfile_lock =
            Regex::new(r"ruby ((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*))[^>]").unwrap();
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

    fn uses_asset_pipeline(&self, app: &App) -> Result<bool> {
        if app.includes_file("Gemfile") {
            return Ok(self.uses_gem_dep(app, "sprockets") || self.uses_gem_dep(app, "propshaft"));
        }
        Ok(false)
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

    fn uses_gem_dep(&self, app: &App, dependency: &str) -> bool {
        ["Gemfile", "Gemfile.lock"]
            .iter()
            .any(|file| app.read_file(file).unwrap_or_default().contains(dependency))
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_gemfile_lock_version() -> Result<()> {
        assert_eq!(
            RubyProvider::get_ruby_version(
                &RubyProvider {},
                &App::new("./examples/ruby")?,
                &Environment::default()
            )?,
            "ruby-3.1.2"
        );

        Ok(())
    }

    #[test]
    fn test_no_version() -> Result<()> {
        assert!(RubyProvider::get_ruby_version(
            &RubyProvider {},
            &App::new("./examples/ruby-no-version")?,
            &Environment::default(),
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_version_file() -> Result<()> {
        assert_eq!(
            RubyProvider::get_ruby_version(
                &RubyProvider {},
                &App::new("./examples/ruby-rails-postgres")?,
                &Environment::default(),
            )?,
            "3.2.1"
        );

        Ok(())
    }

    #[test]
    fn test_version_arg() -> Result<()> {
        assert_eq!(
            RubyProvider::get_ruby_version(
                &RubyProvider {},
                &App::new("./examples/ruby")?,
                &Environment::new(BTreeMap::from([(
                    "NIXPACKS_RUBY_VERSION".to_string(),
                    "3.1.1".to_string()
                )]))
            )?,
            "3.1.1"
        );

        Ok(())
    }
}
