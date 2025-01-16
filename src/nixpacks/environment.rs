use anyhow::Result;
use regex::Regex;
use std::{collections::BTreeMap, env};

pub type EnvironmentVariables = BTreeMap<String, String>;

/// Holds a map of environment variables.
#[derive(Default, Debug)]
pub struct Environment {
    variables: EnvironmentVariables,
}

impl Environment {
    pub fn new(variables: EnvironmentVariables) -> Environment {
        Environment { variables }
    }

    /// Collects all variables from the calling environment.
    pub fn from_envs(envs: Vec<&str>) -> Result<Environment> {
        let mut environment = Environment::default();
        let r = Regex::new(r"([A-Za-z0-9_-]*)(?:=?)([\s\S]*)").unwrap();
        for env in envs {
            let matches = r.captures(env).unwrap();
            if matches.get(2).unwrap().as_str() == "" {
                // No value, pull from the current environment
                let name = matches.get(1).unwrap().as_str();
                if let Ok(value) = env::var(name) {
                    environment.set_variable(name.to_string(), value);
                }
            } else {
                // Use provided name, value pair
                environment.set_variable(
                    matches.get(1).unwrap().as_str().to_string(),
                    matches.get(2).unwrap().as_str().to_string(),
                );
            }
        }

        Ok(environment)
    }

    /// Returns the value of the given variable name, if it exists.
    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(String::as_str)
    }

    /// Returns all the "NIXPACKS_" variables for use in a BuildPlan.
    pub fn get_config_variable(&self, name: &str) -> Option<String> {
        self.get_variable(format!("NIXPACKS_{name}").as_str())
            .map(|var| var.replace('\n', ""))
    }

    /// Checks if the given variable is 1 or true.
    pub fn is_config_variable_truthy(&self, name: &str) -> bool {
        if let Some(var) = self.get_config_variable(name) {
            matches!(var.as_str(), "1" | "true")
        } else {
            false
        }
    }

    /// Store a variable in the Environment.
    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    /// Returns all the variables currently stored in the Environment.
    pub fn get_variable_names(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// Returns a copy of all the environment variables.
    pub fn clone_variables(env: &Environment) -> EnvironmentVariables {
        env.variables.clone()
    }

    /// Add variables to the given Environment.
    pub fn append_variables(env: &Environment, variables: EnvironmentVariables) -> Environment {
        let mut new_env = Environment::new(Environment::clone_variables(env));
        new_env.variables.extend(variables);
        new_env
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
    fn test_create_equals_sign_parsing() {
        let environment = Environment::from_envs(vec!["INVALID=ENV=CONFIG"]).unwrap();
        assert_eq!(environment.get_variable("INVALID"), Some("ENV=CONFIG"));
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
