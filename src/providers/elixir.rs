use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::{Environment, EnvironmentVariables},
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::Result;
use regex::Regex;

enum Mode {
    Compiled,
    Script,
}

enum Framework {
    Nerves,
    Phoenix,
    Vanilla,
}

#[derive(Debug)]
pub struct MixProject {
    pub app_name: Option<String>,
    pub elixir_version: Option<f64>,
    pub is_escript: bool,
    pub is_umbrella: bool,
}

pub struct ElixirProvider {}

const AVAILABLE_ELIXIR_VERSIONS: &[(f64, &str)] = &[
    (1.9, "elixir_1_9"),
    (1.10, "elixir_1_10"),
    (1.11, "elxir_1_11"),
    (1.12, "elixir_1_12"),
    (1.13, "elixir"),
];
const ELIXIR_VERSION_TO_OTP_PKG: &[(f64, &str)] = &[
    (1.9, "erlangR21"),
    (1.10, "erlangR22"),
    (1.11, "erlangR23"),
    (1.12, "erlang"),
    (1.13, "erlangR25"),
];
const DEFAULT_OTP_PKG_NAME: &'static &str = &"erlang";
const DEFAULT_ELIXIR_VERSION: &'static f64 = &1.12;
pub const DEFAULT_ELIXIR_PKG_NAME: &'static &str = &"elixir";

impl Provider for ElixirProvider {
    fn name(&self) -> &str {
        "elixir"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("mix.exs") || app.has_match("*.ex") || app.has_match("*.exs"))
    }

    fn setup(&self, app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        let mix_exs = self.read_mix_exs_if_exists(app)?;
        let mix_project = ElixirProvider::parse_mix_project(mix_exs)?;
        let elixir_nix_pkg = ElixirProvider::get_nix_elixir_pkg(mix_project)?;
        let erlang_nix_pkg = ElixirProvider::get_nix_erlang_pkg(mix_project)?;

        match mix_project {
            Some(MixProject {
                is_escript: true, ..
            }) => {
                let pkgs = vec![Pkg::new(&elixir_nix_pkg), Pkg::new(&erlang_nix_pkg)];

                return Ok(Some(SetupPhase::new(pkgs)));
            }

            _project => Ok(Some(SetupPhase::new(vec![Pkg::new(&elixir_nix_pkg)]))),
        }
    }

    fn install(&self, app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        if app.includes_file("mix.exs") {
            let install_cmd = r#"
                mix local.hex --force --if-missing
                mix local.rebar --force --if-missing
                mix deps.get
                mix deps.compile
            "#;

            return Ok(Some(InstallPhase::new(install_cmd.to_string())));
        }

        Ok(None)
    }

    fn build(&self, app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        let mix_exs = self.read_mix_exs_if_exists(app)?;
        let framework = self.detect_framework(app);
        let mix_project = ElixirProvider::parse_mix_project(mix_exs)?;

        if let Some(project) = mix_project {
            match (project, framework) {
                (
                    MixProject {
                        is_escript: true,
                        app_name: Some(_),
                        ..
                    },
                    _frame,
                ) => return Ok(Some(BuildPhase::new("mix escript.build".to_string()))),

                (
                    MixProject {
                        is_umbrella: true, ..
                    },
                    _framework,
                ) =>
                // TODO
                {
                    return 
                }

                (
                    MixProject {
                        app_name: Some(_), ..
                    },
                    Framework::Nerves,
                ) => return,

                (
                    MixProject {
                        app_name: Some(_), ..
                    },
                    Framework::Phoenix,
                ) => return,

                (
                    MixProject {
                        app_name: Some(_), ..
                    },
                    Framework::Vanilla,
                ) => return,

                (_project, _framework) => return Ok(None),
            }
        }

        Ok(None)
    }

    fn environment_variables(
        &self,
        _app: &App,
        _env: &Environment,
    ) -> Result<Option<EnvironmentVariables>> {
        Ok(Some(EnvironmentVariables::from([(
            "MIX_ENV".to_string(),
            "prod".to_string(),
        )])))
    }
}

impl ElixirProvider {
    fn detect_framework(&self, app: &App) -> Framework {
        let phoenix_re = Regex::new(r"\{\:phoenix\,.+\}").unwrap();
        let has_phoenix_dep = app.find_match(&phoenix_re, "mix.exs").unwrap_or(false);

        if has_phoenix_dep || app.has_match("**/*_web/**") {
            return Framework::Phoenix;
        }

        let nerves_re = Regex::new(r"\{\:nerves\,.+\}").unwrap();
        let has_nerves_dep = app.find_match(&nerves_re, "mix.exs").unwrap_or(false);

        if has_nerves_dep {
            return Framework::Nerves;
        }

        Framework::Vanilla
    }

    fn detect_mode(&self, app: &App) -> Mode {
        if app.includes_file("mix.exs") {
            return Mode::Compiled;
        }

        Mode::Script
    }

    fn read_mix_exs_if_exists(&self, app: &App) -> Result<Option<String>> {
        if app.includes_file("mix.exs") {
            return Ok(Some(app.read_file("mix.exs")?));
        }

        Ok(None)
    }

    fn get_nix_elixir_pkg(mix_project: Option<MixProject>) -> Result<String> {
        let elixir_nix_pkg = mix_project
            .filter(|project| project.elixir_version.is_some())
            .and_then(|project| ElixirProvider::version_to_pkg(&project.elixir_version?).ok()?);

        if let Some(nix_pkg) = elixir_nix_pkg {
            return Ok(nix_pkg);
        }

        Ok(DEFAULT_ELIXIR_PKG_NAME.to_string())
    }

    fn get_nix_erlang_pkg(mix_project: Option<MixProject>) -> Result<String> {
        if let Some(project) = mix_project {
            match project.elixir_version {
                Some(version) => {
                    let matched_pkg =  ELIXIR_VERSION_TO_OTP_PKG
                        .iter()
                        .find(|(v, _)| v == &version)
                        .map(|(_, otp_pkg)| otp_pkg.to_string());

                    if let Some(pkg) = matched_pkg {
                        return Ok(pkg);
                    }
                }

                None => return Ok(DEFAULT_OTP_PKG_NAME.to_string()),
            }
        }

        Ok(DEFAULT_OTP_PKG_NAME.to_string())
    }

    fn version_to_pkg(version: &f64) -> Result<Option<String>> {
        let closest_version = AVAILABLE_ELIXIR_VERSIONS.iter().find(|(v, _)| v >= version);

        if let Some((_, pkg)) = closest_version {
            return Ok(Some(pkg.to_string()));
        }

        Ok(None)
    }

    fn parse_mix_project(mix_exs: Option<String>) -> Result<Option<MixProject>> {
        if let Some(mix_exs) = mix_exs {
            let app_name_re = Regex::new(r"(app:\s)([^s]+),").unwrap();
            let version_re = Regex::new(r"(elixir:\s)([\d\.\d{2}])").unwrap();
            let escript_re = Regex::new(r"(escript:\s)").unwrap();
            let umbrella_re = Regex::new(r"(apps_path:\s)").unwrap();

            let app_name = app_name_re
                .captures(&mix_exs)
                .and_then(|cap| cap.get(2))
                .map(|app_name_cap| app_name_cap.as_str().to_string());
            let elixir_version = version_re
                .captures(&mix_exs)
                .and_then(|cap| cap.get(2))
                .map(|version_cap| version_cap.as_str())
                .map(|version_str| version_str.parse::<f64>().ok())
                .flatten()
                .or(Some(*DEFAULT_ELIXIR_VERSION));
            let is_escript = escript_re.is_match(&mix_exs);
            let is_umbrella = umbrella_re.is_match(&mix_exs);

            let mix_project = MixProject {
                app_name,
                elixir_version,
                is_escript,
                is_umbrella,
            };

            return Ok(Some(mix_project));
        }

        Ok(Some(MixProject {
            app_name: None,
            elixir_version: None,
            is_escript: false,
            is_umbrella: false,
        }))
    }
}
