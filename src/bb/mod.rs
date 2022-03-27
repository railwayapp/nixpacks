use anyhow::{bail, Context, Ok, Result};
use indoc::formatdoc;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
};
use uuid::Uuid;

use crate::builders::Builder;

pub struct AppBuilder<'a> {
    source: PathBuf,
    builder: Option<&'a Box<dyn Builder>>,
}

impl<'a> AppBuilder<'a> {
    pub fn new(source: String) -> AppBuilder<'a> {
        AppBuilder {
            source: fs::canonicalize(PathBuf::from(source)).unwrap(),
            builder: None,
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
                self.builder = Some(builder);
                break;
            }
        }

        match self.builder {
            Some(builder) => println!("  -> Matched builder {}", builder.name()),
            None => bail!("Failed to match a builder"),
        }

        Ok(())
    }

    pub fn build(&self) -> Result<()> {
        println!("\n=== Building ===");

        let nix_expression = self.gen_nix()?;
        println!("  -> Generated Nix expression");

        let dockerfile = self.gen_dockerfile()?;
        println!("  -> Generated Dockerfile");

        let tmp_dir_name = format!("./tmp/{}", Uuid::new_v4());

        println!("  -> Copying source to tmp dir");

        let source = self.source.as_path().to_str().unwrap();
        let mut copy_cmd = Command::new("cp")
            .arg("-R")
            .arg(source)
            .arg(tmp_dir_name.clone())
            .spawn()
            .context("Copying app source to tmp dir")?;
        copy_cmd.wait()?;

        println!("  -> Writing environment.nix");

        let nix_path = PathBuf::from(tmp_dir_name.clone()).join(PathBuf::from("environment.nix"));
        let mut nix_file =
            File::create(nix_path.clone()).context("Creating Nix environment file")?;
        nix_file
            .write_all(nix_expression.as_bytes())
            .context("Unable to write Nix expression")?;

        println!("  -> Writing Dockerfile");

        let dockerfile_path = PathBuf::from(tmp_dir_name.clone()).join(PathBuf::from("Dockerfile"));
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path.clone(), dockerfile).context("Writing Dockerfile")?;

        println!("\nRun:\n  docker build {} -t NAME", tmp_dir_name.as_str());

        Ok(())
    }

    fn gen_nix(&self) -> Result<String> {
        // let build_inputs = &self
        //     .participating_builders
        //     .iter()
        //     .map(|builder| {
        //         let inputs = builder.build_inputs();
        //         inputs
        //     })
        //     .flatten()
        //     .collect::<Vec<String>>();

        let builder = self.builder.expect("Cannot build without builder");

        let pkgs = builder.build_inputs();
        let nix_expression = formatdoc! {"
          {{ pkgs ? import <nixpkgs> {{ }} }}:

          pkgs.mkShell {{ 
            buildInputs = [ {pkgs} ]; 
          }}
        ",
        pkgs=pkgs};

        Ok(nix_expression)
    }

    fn gen_dockerfile(&self) -> Result<String> {
        let builder = self.builder.expect("Cannot build without builder");

        let install_cmd = builder.install_cmd()?.unwrap_or("".to_string());
        let build_cmd = builder.suggested_build_cmd()?.unwrap_or("".to_string());
        let start_cmd = builder.suggested_start_command()?.unwrap_or("".to_string());

        let dockerfile = formatdoc! {"
          FROM nixos/nix

          RUN nix-channel --update

          COPY . /app
          WORKDIR /app

          # Install
          RUN nix-shell environment.nix --pure --run '{install_cmd}'

          # Build
          RUN nix-shell environment.nix --pure --run '{build_cmd}'

          # Start
          CMD nix-shell environment.nix --pure --run '{start_cmd}'
        ",
        install_cmd=install_cmd,
        build_cmd=build_cmd,
        start_cmd=start_cmd};

        Ok(dockerfile)
    }
}
