use crate::{
    nixpacks::{app::App, environment::Environment},
    providers::node::NodeProvider,
};

const MOON_APP_NAME_ENV_VAR: &str = "MOON_APP_NAME";
const MOON_BUILD_TASK_ENV_VAR: &str = "MOON_BUILD_TASK";
const MOON_START_TASK_ENV_VAR: &str = "MOON_START_TASK";

pub struct Moon;

impl Moon {
    pub fn is_moon_repo(app: &App, env: &Environment) -> bool {
        Moon::get_moon_app_name(app, env).is_some() && app.includes_file(".moon/workspace.yml")
    }

    pub fn get_moon_app_name(_app: &App, env: &Environment) -> Option<String> {
        env.get_config_variable(MOON_APP_NAME_ENV_VAR)
    }

    pub fn get_build_cmd(app: &App, env: &Environment) -> String {
        let app_name = Moon::get_moon_app_name(app, env).unwrap();

        let task_name = env
            .get_config_variable(MOON_BUILD_TASK_ENV_VAR)
            .unwrap_or("build".to_string());

        format!(
            "{} @moonrepo/cli run {app_name}:{task_name}",
            NodeProvider::get_package_manager_dlx_command(app)
        )
    }

    pub fn get_start_cmd(app: &App, env: &Environment) -> String {
        let app_name = Moon::get_moon_app_name(app, env).unwrap();

        let task_name = env
            .get_config_variable(MOON_START_TASK_ENV_VAR)
            .unwrap_or("start".to_string());

        format!(
            "{} @moonrepo/cli run {app_name}:{task_name}",
            NodeProvider::get_package_manager_dlx_command(app)
        )
    }
}
