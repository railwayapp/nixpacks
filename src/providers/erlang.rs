use super::Provider;
use crate::nixpacks::{
    app::App,
    environment::Environment,
    nix::pkg::Pkg,
    phase::{BuildPhase, InstallPhase, SetupPhase, StartPhase},
};
use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};

use erl_tokenize::tokens::AtomToken;
use erl_tokenize::Token::{self, Atom};
use erl_tokenize::Tokenizer;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ErlangTasks {}

pub struct ErlangProvider {}

impl Provider for ErlangProvider {
    fn name(&self) -> &str {
        "erlang"
    }

    fn detect(&self, app: &App, _env: &Environment) -> Result<bool> {
        Ok(app.includes_file("rebar.config"))
    }

    fn setup(&self, _app: &App, _env: &Environment) -> Result<Option<SetupPhase>> {
        Ok(Some(SetupPhase::new(vec![
            Pkg::new("erlang"),
            Pkg::new("rebar3"),
        ])))
    }

    fn install(&self, _app: &App, _env: &Environment) -> Result<Option<InstallPhase>> {
        Ok(Some(InstallPhase::new("rebar3 as prod get-deps".into())))
    }

    fn build(&self, _app: &App, _env: &Environment) -> Result<Option<BuildPhase>> {
        Ok(Some(BuildPhase::new("rebar3 as prod release".into())))
    }

    fn start(&self, app: &App, _env: &Environment) -> Result<Option<StartPhase>> {
        if let Some(rebar_file) = app.find_files("rebar.config")?.get(0) {
            if let Ok(Some(name)) =
                get_release_name_from_rebar_config(app, &rebar_file.to_string_lossy())
            {
                Ok(Some(StartPhase::new(format!(
                    "./_build/prod/rel/{}/bin/{} foreground",
                    name, name
                ))))
            } else {
                bail!("Couldn't find release name in rebar.config")
            }
        } else {
            Err(anyhow::anyhow!(
                "Couldn't find release name in rebar.config"
            ))
        }
    }
}

fn get_release_name_from_rebar_config(app: &App, path: &str) -> Result<Option<String>, Error> {
    let src = app.read_file(path)?;
    let tokenizer = Tokenizer::new(&src);
    let release_name_atom = tokenizer
        .filter(|t| matches!(t, Ok(Atom(_))))
        .skip_while(|t| token_is_atom(t, "relx"))
        .skip_while(|t| token_is_atom(t, "release"))
        .nth(1);

    match release_name_atom {
        Some(Ok(Atom(v))) => Ok(Some(v.value().to_string())),
        _ => Ok(None),
    }
}

fn token_is_atom(t: &Result<Token, erl_tokenize::Error>, s: &str) -> bool {
    match t {
        Ok(Atom(v)) => !matches!(v, AtomToken { .. } if v.value() == s),
        _ => true,
    }
}
