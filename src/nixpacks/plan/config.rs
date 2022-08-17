use crate::nixpacks::{environment::Environment, nix::pkg::Pkg};

#[derive(Eq, PartialEq, Clone, Default, Debug)]
pub struct GeneratePlanConfig {
    pub custom_install_cmd: Option<Vec<String>>,
    pub custom_build_cmd: Option<Vec<String>>,
    pub custom_start_cmd: Option<String>,
    pub custom_pkgs: Vec<Pkg>,
    pub custom_libs: Vec<String>,
    pub custom_apt_pkgs: Vec<String>,
    pub pin_pkgs: bool,
    pub install_cache_dirs: Option<Vec<String>>,
    pub build_cache_dirs: Option<Vec<String>>,
}

impl GeneratePlanConfig {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Create configuration from the given environment variables.
    pub fn from_environment(env: &Environment) -> Self {
        Self {
            custom_install_cmd: env.get_config_variable("INSTALL_CMD").map(|cmd| vec![cmd]),
            custom_build_cmd: env.get_config_variable("BUILD_CMD").map(|cmd| vec![cmd]),
            custom_start_cmd: env.get_config_variable("START_CMD"),
            custom_pkgs: env
                .get_config_variable("PKGS")
                .map(|pkg_string| pkg_string.split(' ').map(Pkg::new).collect::<Vec<_>>())
                .unwrap_or_default(),
            custom_apt_pkgs: env
                .get_config_variable("APT_PKGS")
                .map(|apt_string| apt_string.split(' ').map(String::from).collect::<Vec<_>>())
                .unwrap_or_default(),
            custom_libs: env
                .get_config_variable("LIBS")
                .map(|lib_string| lib_string.split(' ').map(String::from).collect::<Vec<_>>())
                .unwrap_or_default(),
            install_cache_dirs: env
                .get_config_variable("INSTALL_CACHE_DIRS")
                .map(|dirs| dirs.split(' ').map(String::from).collect::<Vec<_>>()),
            build_cache_dirs: env
                .get_config_variable("BUILD_CACHE_DIRS")
                .map(|dirs| dirs.split(' ').map(String::from).collect::<Vec<_>>()),
            ..Default::default()
        }
    }

    /// Merge two configurations, preferring the values from the second.
    pub fn merge(c1: &Self, c2: &Self) -> Self {
        Self {
            custom_install_cmd: c1
                .custom_install_cmd
                .clone()
                .or_else(|| c2.custom_install_cmd.clone()),
            custom_build_cmd: c1
                .custom_build_cmd
                .clone()
                .or_else(|| c2.custom_build_cmd.clone()),
            custom_start_cmd: c1
                .custom_start_cmd
                .clone()
                .or_else(|| c2.custom_start_cmd.clone()),
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
            install_cache_dirs: combine_option_vec(
                c1.install_cache_dirs.clone(),
                c2.install_cache_dirs.clone(),
            ),
            build_cache_dirs: combine_option_vec(
                c1.build_cache_dirs.clone(),
                c2.build_cache_dirs.clone(),
            ),
        }
    }
}

fn combine_option_vec<T: Clone>(v1: Option<Vec<T>>, v2: Option<Vec<T>>) -> Option<Vec<T>> {
    match (v1, v2) {
        (Some(v1), Some(v2)) => Some(v1.iter().chain(v2.iter()).cloned().collect()),
        (Some(v1), None) => Some(v1),
        (None, Some(v2)) => Some(v2),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_option_vec() {
        assert_eq!(Some(vec!["a"]), combine_option_vec(Some(vec!["a"]), None));
        assert_eq!(Some(vec!["b"]), combine_option_vec(None, Some(vec!["b"])));
        assert_eq!(
            Some(vec!["a", "b"]),
            combine_option_vec(Some(vec!["a"]), Some(vec!["b"]))
        );
    }

    #[test]
    fn test_config_from_environment_variables() {
        assert_eq!(
            GeneratePlanConfig::default(),
            GeneratePlanConfig::from_environment(&Environment::from_envs(Vec::new()).unwrap())
        );
        assert_eq!(
            GeneratePlanConfig {
                custom_install_cmd: Some(vec!["install".to_string()]),
                custom_build_cmd: Some(vec!["build".to_string()]),
                custom_start_cmd: Some("start".to_string()),
                custom_pkgs: vec![Pkg::new("cowsay")],
                custom_apt_pkgs: vec!["wget".to_string()],
                custom_libs: vec!["openssl".to_string()],
                install_cache_dirs: Some(vec!["install/cache".to_string()]),
                build_cache_dirs: Some(vec!["build/cache".to_string()]),
                pin_pkgs: false
            },
            GeneratePlanConfig::from_environment(
                &Environment::from_envs(vec![
                    "NIXPACKS_INSTALL_CMD=install",
                    "NIXPACKS_BUILD_CMD=build",
                    "NIXPACKS_START_CMD=start",
                    "NIXPACKS_PKGS=cowsay",
                    "NIXPACKS_APT_PKGS=wget",
                    "NIXPACKS_LIBS=openssl",
                    "NIXPACKS_INSTALL_CACHE_DIRS=install/cache",
                    "NIXPACKS_BUILD_CACHE_DIRS=build/cache",
                ])
                .unwrap()
            )
        );
    }

    #[test]
    fn test_config_merge() {
        assert_eq!(
            GeneratePlanConfig {
                custom_install_cmd: Some(vec!["install".to_string()]),
                custom_build_cmd: Some(vec!["build".to_string()]),
                custom_start_cmd: Some("start".to_string()),
                custom_pkgs: vec![Pkg::new("pkg1"), Pkg::new("pkg2")],
                custom_apt_pkgs: vec!["curl".to_string(), "wget".to_string()],
                custom_libs: vec!["openssl".to_string()],
                install_cache_dirs: Some(vec!["install/cache".to_string(), "install2".to_string()]),
                build_cache_dirs: Some(vec!["build/cache".to_string()]),
                pin_pkgs: false
            },
            GeneratePlanConfig::merge(
                &GeneratePlanConfig::from_environment(
                    &Environment::from_envs(vec![
                        "NIXPACKS_INSTALL_CMD=install",
                        "NIXPACKS_START_CMD=start",
                        "NIXPACKS_PKGS=pkg1",
                        "NIXPACKS_APT_PKGS=curl",
                        "NIXPACKS_INSTALL_CACHE_DIRS=install/cache install2",
                    ])
                    .unwrap()
                ),
                &GeneratePlanConfig::from_environment(
                    &Environment::from_envs(vec![
                        "NIXPACKS_BUILD_CMD=build",
                        "NIXPACKS_START_CMD=start",
                        "NIXPACKS_PKGS=pkg2",
                        "NIXPACKS_APT_PKGS=wget",
                        "NIXPACKS_LIBS=openssl",
                        "NIXPACKS_BUILD_CACHE_DIRS=build/cache",
                    ])
                    .unwrap()
                )
            )
        );
    }
}
