use std::collections::HashMap;

pub type EnvironmentVariables = HashMap<String, String>;

#[derive(Default, Debug)]
pub struct Environment {
    variables: EnvironmentVariables,
}

impl Environment {
    pub fn new(variables: EnvironmentVariables) -> Environment {
        Environment { variables }
    }

    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
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
        assert_eq!(
            environment.get_variable("hello"),
            Some(&"world".to_string())
        );
    }
}
