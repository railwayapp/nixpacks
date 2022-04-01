use anyhow::{bail, Context, Ok, Result};
use colored::Colorize;
use indoc::formatdoc;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
};
use uuid::Uuid;
pub mod app;
pub mod logger;

use crate::providers::Provider;

use self::{app::App, logger::Logger};

static NIXPKGS_ARCHIVE: &'static str =
    "https://github.com/NixOS/nixpkgs/archive/30d3d79b7d3607d56546dd2a6b49e156ba0ec634.tar.gz";

pub struct AppBuilder<'a> {
    name: Option<String>,
    app: &'a App,
    logger: &'a Logger,
    custom_build_cmd: Option<String>,
    custom_start_cmd: Option<String>,
    pkgs: Vec<String>,
    provider: Option<&'a dyn Provider>,
}

impl<'a> AppBuilder<'a> {
    pub fn new(
        name: Option<String>,
        app: &'a App,
        logger: &'a Logger,
        custom_build_cmd: Option<String>,
        custom_start_cmd: Option<String>,
        pkgs: Vec<String>,
    ) -> Result<AppBuilder<'a>> {
        Ok(AppBuilder {
            name,
            app,
            logger,
            custom_build_cmd,
            custom_start_cmd,
            pkgs,
            provider: None,
        })
    }

    pub fn detect(&mut self, providers: Vec<&'a dyn Provider>) -> Result<()> {
        self.logger.log_section("Detecting");

        for provider in providers {
            let matches = provider.detect(self.app)?;
            if matches {
                self.provider = Some(provider);
                break;
            }
        }

        match self.provider {
            Some(provider) => self
                .logger
                .log_step(&format!("Matched provider {}", provider.name()).blue()),
            None => {
                // If no builder is found, only fail if there is no start command
                if self.custom_start_cmd.is_none() {
                    bail!("Failed to match a provider")
                }

                self.logger
                    .log_step("No provider matched. Using start command")
            }
        }

        Ok(())
    }

    pub fn build(&self) -> Result<()> {
        self.logger.log_section("Building");

        let nix_expression = self.gen_nix()?;
        self.logger.log_step("Generated Nix expression");

        let dockerfile = self.gen_dockerfile()?;
        self.logger.log_step("Generated Dockerfile");

        let id = Uuid::new_v4();
        let tmp_dir_name = format!("./tmp/{}", id);

        self.logger.log_step("Copying source to tmp dir");

        let source = self.app.source.as_path().to_str().unwrap();
        let mut copy_cmd = Command::new("cp")
            .arg("-R")
            .arg(source)
            .arg(tmp_dir_name.clone())
            .spawn()?;
        copy_cmd.wait().context("Copying app source to tmp dir")?;

        self.logger.log_step("Writing environment.nix");

        let nix_path = PathBuf::from(tmp_dir_name.clone()).join(PathBuf::from("environment.nix"));
        let mut nix_file = File::create(nix_path).context("Creating Nix environment file")?;
        nix_file
            .write_all(nix_expression.as_bytes())
            .context("Unable to write Nix expression")?;

        self.logger.log_step("Writing Dockerfile");

        let dockerfile_path = PathBuf::from(tmp_dir_name.clone()).join(PathBuf::from("Dockerfile"));
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path, dockerfile).context("Writing Dockerfile")?;

        // println!(
        //     "\nRun:\n  docker build {} -t {}",
        //     tmp_dir_name.as_str(),
        //     self.name.clone().unwrap_or(id.to_string())
        // );

        self.logger.log_step("Building image");

        let name = self.name.clone().unwrap_or_else(|| id.to_string());

        let mut docker_build_cmd = Command::new("docker")
            .arg("build")
            .arg(tmp_dir_name.as_str())
            .arg("-t")
            .arg(name.clone())
            .spawn()?;

        let build_result = docker_build_cmd.wait().context("Building image")?;

        if !build_result.success() {
            bail!("Docker build failed")
        }

        self.logger.log_section("Successfully Built!");

        println!("\nRun:");
        println!("  docker run {}", name);

        Ok(())
    }

    pub fn gen_nix(&self) -> Result<String> {
        // let build_inputs = &self
        //     .participating_builders
        //     .iter()
        //     .map(|builder| {
        //         let inputs = builder.build_inputs();
        //         inputs
        //     })
        //     .flatten()
        //     .collect::<Vec<String>>();

        let user_pkgs = self
            .pkgs
            .iter()
            .map(|s| format!("pkgs.{}", s))
            .collect::<Vec<String>>()
            .join(" ");

        let pkgs = match self.provider {
            Some(provider) => format!("{} {}", provider.pkgs(self.app), user_pkgs),
            None => user_pkgs,
        };

        // let nix_expression = formatdoc! {"
        //   {{ pkgs ? import <nixpkgs> {{ }} }}:

        //   pkgs.mkShell {{
        //     buildInputs = [ {pkgs} ];
        //   }}
        // ",
        // pkgs=pkgs};

        let nix_expression = formatdoc! {"
          with import (fetchTarball \"{nixpkgs}\") {{ }}; [ {pkgs} ]
        ",
        nixpkgs=NIXPKGS_ARCHIVE,
        pkgs=pkgs};

        Ok(nix_expression)
    }

    pub fn gen_dockerfile(&self) -> Result<String> {
        // let builder = self.builder.expect("Cannot build without builder");

        let install_cmd = match self.provider {
            Some(provider) => provider
                .install_cmd(self.app)?
                .unwrap_or_else(|| "".to_string()),
            None => "".to_string(),
        };

        let suggested_build_cmd = match self.provider {
            Some(provider) => provider
                .suggested_build_cmd(self.app)?
                .unwrap_or_else(|| "".to_string()),
            None => "".to_string(),
        };
        let build_cmd = self.custom_build_cmd.clone().unwrap_or(suggested_build_cmd);

        let suggested_start_cmd = match self.provider {
            Some(provider) => provider
                .suggested_start_command(self.app)?
                .unwrap_or_else(|| "".to_string()),
            None => "".to_string(),
        };
        let procfile_cmd = self.parse_procfile()?;
        let start_cmd = procfile_cmd
            .unwrap_or_else(|| self.custom_start_cmd.clone().unwrap_or(suggested_start_cmd));

        let dockerfile = formatdoc! {"
          FROM nixos/nix

          RUN nix-channel --update

          COPY . /app
          WORKDIR /app

          # Load Nix environment
          RUN nix-env -if environment.nix

          # Install
          RUN {install_cmd}

          # Build
          RUN {build_cmd}

          # Start
          CMD {start_cmd}
        ",
        install_cmd=install_cmd,
        build_cmd=build_cmd,
        start_cmd=start_cmd};

        Ok(dockerfile)
    }

    fn parse_procfile(&self) -> Result<Option<String>> {
        if self.app.includes_file("Procfile") {
            let contents = self.app.read_file("Procfile")?;

            // Better error handling
            if contents.starts_with("web: ") {
                return Ok(Some(contents.replace("web: ", "")));
            }

            Ok(None)
        } else {
            Ok(None)
        }
    }
}
