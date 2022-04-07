use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Pkg {
    pub name: String,
    pub overrides: HashMap<String, String>,
}

impl Pkg {
    pub fn new(name: &str) -> Pkg {
        Pkg {
            name: name.to_string(),
            overrides: HashMap::default(),
        }
    }

    pub fn to_nix_string(&self) -> String {
        if self.overrides.is_empty() {
            self.name.clone()
        } else {
            let override_string = self
                .overrides
                .iter()
                .map(|(name, value)| format!("{} = {};", name, value))
                .collect::<Vec<_>>()
                .join(" ");
            format!("({}.override {{ {} }})", self.name, override_string)
        }
    }

    pub fn set_override(mut self, name: &str, pkg: &str) -> Self {
        self.overrides.insert(name.to_string(), pkg.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pkg_to_string() {
        assert_eq!(Pkg::new("cowsay").to_nix_string(), "cowsay".to_string());
    }

    #[test]
    fn test_pkg_single_override_to_string() {
        assert_eq!(
            Pkg::new("cowsay")
                .set_override("hello", "hello_1.1")
                .to_nix_string(),
            "(cowsay.override { hello = hello_1.1; })".to_string()
        );
    }
}
