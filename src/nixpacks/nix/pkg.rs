use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Pkg {
    pub name: String,
    pub overlay: Option<String>,
    pub overrides: Option<HashMap<String, String>>,
}

impl Pkg {
    pub fn new(name: &str) -> Pkg {
        Pkg {
            name: name.to_string(),
            overrides: None,
            overlay: None,
        }
    }

    pub fn to_nix_string(&self) -> String {
        match &self.overrides {
            Some(overrides) => {
                let override_string = overrides
                    .iter()
                    .map(|(name, value)| format!("{name} = {value};"))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("({}.override {{ {override_string} }})", self.name)
            }
            None => self.name.clone(),
        }
    }

    #[must_use]
    pub fn set_override(mut self, name: &str, pkg: &str) -> Self {
        if let Some(mut overrides) = self.overrides {
            overrides.insert(name.to_string(), pkg.to_string());
            self.overrides = Some(overrides);
        } else {
            self.overrides = Some(HashMap::from([(name.to_string(), pkg.to_string())]));
        }

        self
    }

    #[must_use]
    pub fn from_overlay(mut self, overlay: &str) -> Self {
        self.overlay = Some(overlay.to_string());
        self
    }

    pub fn to_pretty_string(&self) -> String {
        match &self.overrides {
            Some(overrides) => {
                let override_string = overrides
                    .iter()
                    .map(|(name, value)| format!("{name} = {value}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {{ {override_string} }}", self.name)
            }
            None => self.name.clone(),
        }
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
