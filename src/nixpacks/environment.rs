use anyhow::{bail, Result};
use std::{collections::HashMap, env};

pub type EnvironmentVariables = HashMap<String, String>;

#[derive(Default, Debug)]
pub struct Environment {
    variables: EnvironmentVariables,
}

impl Environment {
    pub fn new(variables: EnvironmentVariables) -> Environment {
        Environment { variables }
    }

    pub fn from_envs(envs: Vec<&str>) -> Result<Environment> {
        let mut environment = Environment::default();
        for env in envs {
            let v: Vec<&str> = env.split('=').collect();
            if v.len() == 1 {
                // Pull the variable from the current environment
                let name = v[0];
                if let Ok(value) = env::var(name) {
                    // Variable is set
                    environment.set_variable(name.to_string(), value);
                }
            } else if v.len() > 2 {
                bail!("Unable to parse variable string");
            } else {
                // Use provided name, value pair
                environment.set_variable(v[0].to_string(), v[1].to_string());
            }
        }

        Ok(environment)
    }

    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(String::as_str)
    }

    pub fn get_config_variable(&self, name: &str) -> Option<String> {
        self.get_variable(format!("NIXPACKS_{}", name).as_str())
            .map(|var| var.replace('\n', ""))
    }

    pub fn is_config_variable_truthy(&self, name: &str) -> bool {
        if let Some(var) = self.get_config_variable(name) {
            matches!(var.as_str(), "1" | "true")
        } else {
            false
        }
    }

    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    pub fn get_variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    pub fn clone_variables(env: &Environment) -> EnvironmentVariables {
        env.variables.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::Environment;

    #[test]
    fn set_and_get_variables() {
        let mut environment = Environment::default();
        assert!(environment.get_variable("hello").is_none());
        environment.set_variable("hello".to_string(), "world".to_string());
        assert_eq!(environment.get_variable("hello"), Some("world"));
    }

    #[test]
    fn test_environment_variable_parsing() {
        let environment =
            Environment::from_envs(vec!["HELLO=world", "CARGO_PKG_NAME", "NON_EXISTANT"]).unwrap();
        assert_eq!(environment.get_variable("HELLO"), Some("world"));
        assert_eq!(environment.get_variable("CARGO_PKG_NAME"), Some("nixpacks"));
        assert!(environment.get_variable("NON_EXISTANT").is_none());
    }

    #[test]
    fn test_create_invalid_environment() {
        assert!(Environment::from_envs(vec!["INVALID=ENV=CONFIG"]).is_err());
    }

    #[test]
    fn test_get_config_variable() {
        let mut environment = Environment::default();
        environment.set_variable("NIXPACKS_HELLO".to_string(), "world".to_string());
        assert_eq!(
            environment.get_config_variable("HELLO"),
            Some("world".to_string())
        );
    }

    #[test]
    fn test_get_config_variable_truthy() {
        let mut environment = Environment::default();

        environment.set_variable("NIXPACKS_YES".to_string(), "1".to_string());
        environment.set_variable("NIXPACKS_NO".to_string(), "0".to_string());

        assert!(environment.is_config_variable_truthy("YES"));
        assert!(!environment.is_config_variable_truthy("NO"));
    }

    #[test]
    fn test_get_config_variable_strips_newlines() {
        let mut environment = Environment::default();
        environment.set_variable("NIXPACKS_BUILD_CMD".to_string(), "hello\nworld".to_string());
        assert_eq!(
            environment.get_config_variable("BUILD_CMD"),
            Some("helloworld".to_string())
        );
    }
}
