use anyhow::{Context, Ok, Result};
use indoc::formatdoc;
use std::{
    fs::{self, File},
    path::PathBuf,
    process::Command,
};
use uuid::Uuid;

use crate::builders::Builder;

// #[derive(Error, Debug)]
// pub enum AppBuilderError {
//     #[error("Invalid app source directory: {0}")]
//     InvalidAppSource(String),
// }

pub struct AppBuilder<'a> {
    source: PathBuf,
    participating_builders: Vec<&'a Box<dyn Builder>>,
}

impl<'a> AppBuilder<'a> {
    pub fn new(source: String) -> AppBuilder<'a> {
        AppBuilder {
            source: fs::canonicalize(PathBuf::from(source)).unwrap(),
            participating_builders: Vec::new(),
        }
    }

    pub fn detect(&mut self, builders: &'a Vec<Box<dyn Builder>>) -> Result<()> {
        println!("=== Detecting ===");

        let dir =
            fs::read_dir(self.source.clone()).context("Failed to read app source directory")?;

        let paths: Vec<PathBuf> = dir.map(|path| path.unwrap().path()).collect();

        for builder in builders {
            let matches = builder.detect(paths.clone())?;
            if matches {
                self.participating_builders.push(builder);
            }
        }

        println!("  Participating builders");
        for builder in &self.participating_builders {
            println!("    -> {}", builder.name());
        }

        Ok(())
    }

    pub fn build(&self) -> Result<()> {
        println!("\n=== Building ===");

        let nix_expression = self.gen_nix()?;
        println!("  Generated Nix expression");

        let dockerfile = self.gen_dockerfile()?;
        println!("  Generated Dockerfile");

        let tmp_dir_name = format!("./tmp/{}", Uuid::new_v4());

        println!("  Copying source to tmp dir");

        let source = self.source.as_path().to_str().unwrap();
        Command::new("cp")
            .arg("-R")
            .arg(source)
            .arg(tmp_dir_name.clone())
            .spawn()
            .context("Copying app source to tmp dir")?;

        println!("  Writing environment.nix");

        let nix_path = self.source.clone().join(PathBuf::from("environment.nix"));
        File::create(nix_path.clone()).context("Creating Nix environment file")?;
        fs::write(nix_path, nix_expression).context("Writing Nix environment")?;

        println!("  Writing Dockerfile");

        let dockerfile_path = self.source.clone().join(PathBuf::from("Dockerfile"));
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path.clone(), dockerfile).context("Writing Dockerfile")?;

        println!("Run `docker build {} -t NAME", tmp_dir_name.as_str());

        Ok(())
    }

    fn gen_nix(&self) -> Result<String> {
        let build_inputs = &self
            .participating_builders
            .iter()
            .map(|builder| {
                let inputs = builder.build_inputs();
                inputs
            })
            .flatten()
            .collect::<Vec<String>>();

        let nix_expression = formatdoc! {"
          {{ pkgs ? import <nixpkgs> {{ }} }}:

          pkgs.mkShell {{ 
            buildInputs = [ {pkgs} ]; 
          }}
        ",
        pkgs = build_inputs.join(" ")};

        Ok(nix_expression)
    }

    fn gen_dockerfile(&self) -> Result<String> {
        // Install commands for all participating builders
        let mut install_cmds: Vec<String> = Vec::new();
        for builder in &self.participating_builders {
            match builder.install_cmd()? {
                Some(cmd) => install_cmds.push(cmd),
                None => {}
            }
        }

        // Build command of the last builder
        let mut suggested_build_cmd: Option<String> = None;
        for builder in &self.participating_builders {
            match builder.suggested_build_cmd()? {
                Some(cmd) => suggested_build_cmd = Some(cmd),
                None => {}
            }
        }

        // Start command of the last builder
        let mut suggested_start_cmd: Option<String> = None;
        for builder in &self.participating_builders {
            match builder.suggested_start_command()? {
                Some(cmd) => suggested_start_cmd = Some(cmd),
                None => {}
            }
        }

        let install_cmd = install_cmds
            .iter()
            .map(|cmd| format!("RUN nix-shell environment.nix --run '{}'", cmd))
            .collect::<Vec<String>>()
            .join("\n");

        let build_cmd = suggested_build_cmd.unwrap_or("".to_string());
        let start_cmd = suggested_start_cmd.unwrap_or("".to_string());

        let dockerfile = formatdoc! {"
          FROM nixos/nix

          RUN nix-channel --update

          COPY . /app
          WORKDIR /app

          # Install
          {install_cmd}

          # Build
          RUN nix-shell environment.nix --run '{build_cmd}'

          # Start
          CMD nix-shell environment.nix --run '{start_cmd}'
        ",
        install_cmd=install_cmd,
        build_cmd=build_cmd,
        start_cmd=start_cmd};

        Ok(dockerfile)
    }
}
