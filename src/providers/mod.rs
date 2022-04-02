use crate::nixpacks::app::App;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod go;
pub mod npm;
pub mod rust;
pub mod yarn;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Pkg {
    pub name: String,
}

impl Pkg {
    pub fn new(name: &str) -> Pkg {
        Pkg {
            name: name.to_string(),
        }
    }
}

// impl Serialize for Pkg {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         serializer.serialize_str(self.name.as_str())
//     }
// }

pub trait Provider {
    fn name(&self) -> &str;
    fn detect(&self, app: &App) -> Result<bool>;
    fn pkgs(&self, app: &App) -> Vec<Pkg>;
    fn install_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_build_cmd(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }
    fn suggested_start_command(&self, _app: &App) -> Result<Option<String>> {
        Ok(None)
    }
}
