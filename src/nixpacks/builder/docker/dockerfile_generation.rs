use crate::nixpacks::{
    environment::Environment,
    images::DEFAULT_BASE_IMAGE,
    nix,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{Context, Ok, Result};
use indoc::formatdoc;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use super::{utils, DockerBuilderOptions};

pub trait DockerfileGenerator {
    fn generate_dockerfile(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
    ) -> Result<String>;
    fn write_supporting_files(
        &self,
        _options: &DockerBuilderOptions,
        _env: &Environment,
        _dest: &str,
    ) -> Result<()> {
        Ok(())
    }
}

pub static APP_DIR: &str = "/app/";

impl DockerfileGenerator for BuildPlan {
    fn generate_dockerfile(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
    ) -> Result<String> {
        let plan = self;

        let variables = plan.variables.clone().unwrap_or_default();
        let args_string = if !variables.is_empty() {
            format!(
                "ARG {}\nENV {}",
                // Pull the variables in from docker `--build-arg`
                variables
                    .iter()
                    .map(|var| var.0.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                // Make the variables available at runtime
                variables
                    .iter()
                    .map(|var| format!("{}=${}", var.0, var.0))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        } else {
            "".to_string()
        };

        let dockerfile_phases = plan
            .get_sorted_phases()?
            .into_iter()
            .map(|phase| {
                let phase_dockerfile = phase
                    .generate_dockerfile(options, env)
                    .context(format!("Generating Dockerfile for phase {}", phase.name))?;

                match phase.name.as_str() {
                    // We want to load the variables immediately after the setup phase
                    "setup" => Ok(format!(
                        "{phase_dockerfile}\n# load variables\n{args_string}\n"
                    )),
                    _ => Ok(phase_dockerfile),
                }
            })
            .collect::<Result<Vec<_>>>();
        let dockerfile_phases_str = dockerfile_phases?.join("\n");

        let start_phase_str = plan
            .start_phase
            .clone()
            .unwrap_or_default()
            .generate_dockerfile(options, env)?;

        let base_image = plan.build_image.clone();

        let dockerfile = formatdoc! {"
            FROM {base_image}
            WORKDIR {APP_DIR}

            {dockerfile_phases_str}

            {start_phase_str}
        "};

        Ok(dockerfile)
    }

    fn write_supporting_files(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
        dest: &str,
    ) -> Result<()> {
        self.write_assets(self, dest).context("Writing assets")?;

        for phase in self.get_sorted_phases()? {
            phase
                .write_supporting_files(options, env, dest)
                .context(format!("Writing files for phase {}", phase.name))?;
        }

        Ok(())
    }
}

impl BuildPlan {
    fn write_assets(&self, plan: &BuildPlan, dest: &str) -> Result<()> {
        if let Some(assets) = &plan.static_assets {
            if !assets.is_empty() {
                let static_assets_path = PathBuf::from(dest).join(PathBuf::from("assets"));
                fs::create_dir_all(&static_assets_path).context("Creating static assets folder")?;

                for (name, content) in assets {
                    let path = Path::new(&static_assets_path).join(name);
                    let parent = path.parent().unwrap();
                    fs::create_dir_all(parent)
                        .context(format!("Creating parent directory for {}", name))?;
                    let mut file =
                        File::create(path).context(format!("Creating asset file for {name}"))?;
                    file.write_all(content.as_bytes())
                        .context(format!("Writing asset {name}"))?;
                }
            }
        }

        Ok(())
    }
}

impl DockerfileGenerator for StartPhase {
    fn generate_dockerfile(
        &self,
        _options: &DockerBuilderOptions,
        _env: &Environment,
    ) -> Result<String> {
        // TODO: Handle run images

        if let Some(cmd) = &self.cmd {
            Ok(formatdoc! {"# start
            CMD {cmd}"})
        } else {
            Ok("".to_string())
        }
    }
}

impl DockerfileGenerator for Phase {
    fn generate_dockerfile(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
    ) -> Result<String> {
        let phase = self;

        let cache_key = if !options.no_cache && !env.is_config_variable_truthy("NO_CACHE") {
            options.cache_key.clone()
        } else {
            None
        };

        // Install nix packages and libraries
        let install_nix_pkgs_str = if phase.nix_pkgs.is_some() || phase.nix_libraries.is_some() {
            let nix_file_name = format!("{}.nix", phase.name);
            format!("COPY {nix_file_name} .\nRUN nix-env -if {nix_file_name}")
        } else {
            "".to_string()
        };

        // Install apt packages
        let apt_pkgs = phase.apt_pkgs.clone().unwrap_or_default();
        let apt_pkgs_str = if !apt_pkgs.is_empty() {
            format!(
                "RUN apt-get update && apt-get install -y {}",
                apt_pkgs.join(" ")
            )
        } else {
            "".to_string()
        };

        // Copy over app files
        let phase_files = match (phase.name.as_str(), &phase.only_include_files) {
            (_, Some(files)) => files.clone(),
            // Special case for the setup phase, which has no files
            ("setup", None) => vec![],
            _ => vec![".".to_string()],
        };
        let phase_copy_cmd = utils::get_copy_command(&phase_files, APP_DIR);

        let cache_mount = utils::get_cache_mount(&cache_key, &phase.cache_directories);
        let cmds_str = phase
            .cmds
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|s| format!("RUN {} {}", cache_mount, s))
            .collect::<Vec<_>>()
            .join("\n");

        let dockerfile_stmts = vec![install_nix_pkgs_str, apt_pkgs_str, phase_copy_cmd, cmds_str]
            .into_iter()
            .filter(|stmt| !stmt.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let dockerfile = formatdoc! {"
            # {name} phase
            {dockerfile_stmts}
        ", name=phase.name};

        Ok(dockerfile)
    }

    fn write_supporting_files(
        &self,
        _options: &DockerBuilderOptions,
        _env: &Environment,
        dest: &str,
    ) -> Result<()> {
        if !self.nix_pkgs.clone().unwrap_or_default().is_empty()
            || !self.nix_libraries.clone().unwrap_or_default().is_empty()
        {
            // Write the Nix expressions to the output directory
            let nix_file_name = format!("{}.nix", self.name);
            let nix_path = PathBuf::from(dest).join(PathBuf::from(nix_file_name));
            let nix_expression = nix::create_nix_expression(self);

            let mut nix_file = File::create(nix_path).context("Creating Nix environment file")?;
            nix_file
                .write_all(nix_expression.as_bytes())
                .context("Unable to write Nix expression")?;
        }

        Ok(())
    }
}
