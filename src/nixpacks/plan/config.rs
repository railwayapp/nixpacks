use crate::nixpacks::{environment::Environment, nix::pkg::Pkg};

#[derive(Clone, Default, Debug)]
pub struct GeneratePlanConfig {
    pub custom_install_cmd: Option<Vec<String>>,
    pub custom_build_cmd: Option<Vec<String>>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
    pub custom_libs: Vec<String>,
    pub custom_apt_pkgs: Vec<String>,
    pub pin_pkgs: bool,
}

impl GeneratePlanConfig {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Create configuration from the given environment variables.
    pub fn from_environment(environment: &Environment) -> Self {
        Self {
            custom_install_cmd: environment
                .get_config_variable("INSTALL_CMD")
                .map(|cmd| vec![cmd]),
            custom_build_cmd: environment
                .get_config_variable("BUILD_CMD")
                .map(|cmd| vec![cmd]),
            custom_start_cmd: environment.get_config_variable("START_CMD"),
            custom_pkgs: environment
                .get_config_variable("PKGS")
                .map(|pkg_string| pkg_string.split(' ').map(Pkg::new).collect::<Vec<_>>())
                .unwrap_or_default(),
            custom_apt_pkgs: environment
                .get_config_variable("APT_PKGS")
                .map(|apt_string| apt_string.split(' ').map(String::from).collect::<Vec<_>>())
                .unwrap_or_default(),
            custom_libs: environment
                .get_config_variable("LIBS")
                .map(|lib_string| lib_string.split(' ').map(String::from).collect::<Vec<_>>())
                .unwrap_or_default(),
            ..Default::default()
        }
    }

    /// Merge two configurations, preferring the values from the second.
    pub fn merge(c1: &Self, c2: &Self) -> Self {
        Self {
            custom_install_cmd: c1
                .custom_install_cmd
                .clone()
                .or(c2.custom_install_cmd.clone()),
            custom_build_cmd: c1.custom_build_cmd.clone().or(c2.custom_build_cmd.clone()),
            custom_start_cmd: c1.custom_start_cmd.clone().or(c2.custom_start_cmd.clone()),
            custom_pkgs: c1
                .custom_pkgs
                .iter()
                .chain(c2.custom_pkgs.iter())
                .cloned()
                .collect(),
            custom_libs: c1
                .custom_libs
                .iter()
                .chain(c2.custom_libs.iter())
                .cloned()
                .collect(),
            custom_apt_pkgs: c1
                .custom_apt_pkgs
                .iter()
                .chain(c2.custom_apt_pkgs.iter())
                .cloned()
                .collect(),
            pin_pkgs: c1.pin_pkgs || c2.pin_pkgs,
        }
    }
}

// TODO: Tests
