use super::{utils, DockerBuilderOptions};
use crate::nixpacks::{
    app,
    environment::Environment,
    nix,
    plan::{
        phase::{Phase, StartPhase},
        BuildPlan,
    },
};
use anyhow::{Context, Ok, Result};
use indoc::formatdoc;
use path_slash::PathBufExt;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

const NIXPACKS_OUTPUT_DIR: &str = ".nixpacks";
pub const APP_DIR: &str = "/app/";

#[derive(Debug, Clone)]
pub struct OutputDir {
    pub root: PathBuf,
    pub asset_root: PathBuf,
    pub is_temp: bool,
}

impl OutputDir {
    pub fn new(root: PathBuf, is_temp: bool) -> Result<Self> {
        let root = root;
        let asset_root = PathBuf::from(NIXPACKS_OUTPUT_DIR);

        Ok(Self {
            root,
            asset_root,
            is_temp,
        })
    }

    pub fn from(root: &str, is_temp: bool) -> Result<Self> {
        Self::new(PathBuf::from(root), is_temp)
    }

    /// Ensure that the output directory and all necessary subdirectories exist.
    pub fn ensure_output_exists(&self) -> Result<()> {
        // Create the root output directory if needed
        if fs::metadata(&self.root).is_err() {
            fs::create_dir_all(&self.root).context("Creating output directory")?;
        }

        // Create the assets directory if it doesn't exist
        let full_asset_path = self.root.join(self.asset_root.clone());
        if fs::metadata(&full_asset_path).is_err() {
            fs::create_dir_all(&full_asset_path).context("Creating assets directory")?;
        }

        Ok(())
    }

    pub fn get_relative_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.asset_root.join(path)
    }

    pub fn get_absolute_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root.join(self.get_relative_path(path))
    }
}

impl Default for OutputDir {
    fn default() -> Self {
        Self::from(".", false).unwrap()
    }
}

pub trait DockerfileGenerator {
    fn generate_dockerfile(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
        output: &OutputDir,
    ) -> Result<String>;
    fn write_supporting_files(
        &self,
        _options: &DockerBuilderOptions,
        _env: &Environment,
        _output: &OutputDir,
    ) -> Result<()> {
        Ok(())
    }
}

impl DockerfileGenerator for BuildPlan {
    fn generate_dockerfile(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
        output: &OutputDir,
    ) -> Result<String> {
        let plan = self;

        let variables = plan.variables.clone().unwrap_or_default();
        let args_string = if variables.is_empty() {
            "".to_string()
        } else {
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
        };

        let static_assets = plan.static_assets.clone().unwrap_or_default();
        let assets_copy_cmd = if static_assets.is_empty() {
            "".to_string()
        } else {
            let rel_assets_path = output.get_relative_path("assets");
            let rel_assets_slash_path = rel_assets_path
                .to_slash()
                .context("Failed to convert nix file path to slash path.")?;
            format!("COPY {} {}", rel_assets_slash_path, app::ASSETS_DIR)
        };

        let dockerfile_phases = plan
            .get_sorted_phases()?
            .into_iter()
            .map(|phase| {
                let phase_dockerfile = phase
                    .generate_dockerfile(options, env, output)
                    .context(format!("Generating Dockerfile for phase {}", phase.name))?;

                match phase.name.as_str() {
                    // We want to load the variables immediately after the setup phase
                    "setup" => Ok(format!(
                        "{}\n# load variables\n{}\n",
                        phase_dockerfile, args_string
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
            .generate_dockerfile(options, env, output)?;

        let base_image = plan.build_image.clone();

        let dockerfile = formatdoc! {"
            FROM {base_image}

            ENTRYPOINT [\"/bin/bash\", \"-l\", \"-c\"]
            WORKDIR {APP_DIR}

            {assets_copy_cmd}

            {dockerfile_phases_str}

            {start_phase_str}
        ", 
        base_image=base_image,
        APP_DIR=APP_DIR,
        assets_copy_cmd=assets_copy_cmd,
        dockerfile_phases_str=dockerfile_phases_str,
        start_phase_str=start_phase_str};

        Ok(dockerfile)
    }

    fn write_supporting_files(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
        output: &OutputDir,
    ) -> Result<()> {
        self.write_assets(self, output).context("Writing assets")?;

        for phase in self.get_sorted_phases()? {
            phase
                .write_supporting_files(options, env, output)
                .context(format!("Writing files for phase {}", phase.name))?;
        }

        Ok(())
    }
}

impl BuildPlan {
    fn write_assets(&self, plan: &BuildPlan, output: &OutputDir) -> Result<()> {
        if let Some(assets) = &plan.static_assets {
            if !assets.is_empty() {
                let static_assets_path = output.get_absolute_path("assets");
                fs::create_dir_all(&static_assets_path).context("Creating static assets folder")?;

                for (name, content) in assets {
                    let path = Path::new(&static_assets_path).join(name);
                    let parent = path.parent().unwrap();
                    fs::create_dir_all(parent)
                        .context(format!("Creating parent directory for {}", name))?;
                    let mut file =
                        File::create(path).context(format!("Creating asset file for {}", name))?;
                    file.write_all(content.as_bytes())
                        .context(format!("Writing asset {}", name))?;
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
        _output: &OutputDir,
    ) -> Result<String> {
        let start_cmd = match &self.cmd {
            Some(cmd) => utils::get_exec_command(cmd),
            None => "".to_string(),
        };

        let dockerfile: String = match &self.run_image {
            Some(run_image) => {
                let copy_cmd = utils::get_copy_from_command(
                    "0",
                    &self.only_include_files.clone().unwrap_or_default(),
                    APP_DIR,
                );

                // RUN true to prevent a Docker bug https://github.com/moby/moby/issues/37965#issuecomment-426853382
                formatdoc! {"
                  # start
                  FROM {run_image}
                  WORKDIR {APP_DIR}
                  COPY --from=0 /etc/ssl/certs /etc/ssl/certs
                  RUN true
                  {copy_cmd}
                  {start_cmd}
                ",
                run_image=run_image,
                APP_DIR=APP_DIR,
                copy_cmd=copy_cmd,
                start_cmd=start_cmd,}
            }
            None => {
                formatdoc! {"
                  # start
                  {}
                ",
                start_cmd}
            }
        };

        Ok(dockerfile)
    }
}

impl DockerfileGenerator for Phase {
    fn generate_dockerfile(
        &self,
        options: &DockerBuilderOptions,
        env: &Environment,
        output: &OutputDir,
    ) -> Result<String> {
        let phase = self;

        let cache_key = if !options.no_cache && !env.is_config_variable_truthy("NO_CACHE") {
            options.cache_key.clone()
        } else {
            None
        };

        // Ensure paths are available in the environment
        let (build_path, run_path) = if let Some(paths) = &phase.paths {
            let joined_paths = paths.join(":");
            (
                format!("ENV PATH {}:$PATH", joined_paths),
                format!(
                    "RUN printf '\\nPATH={}:$PATH' >> /root/.profile",
                    joined_paths
                ),
            )
        } else {
            ("".to_string(), "".to_string())
        };

        // Install nix packages and libraries
        let install_nix_pkgs_str = if self.uses_nix() {
            let nix_file = output.get_relative_path(format!("{}.nix", phase.name));

            let nix_file_path = nix_file
                .to_slash()
                .context("Failed to convert nix file path to slash path.")?;
            format!(
                "COPY {nix_file_path} {nix_file_path}\nRUN nix-env -if {nix_file_path}",
                nix_file_path = nix_file_path
            )
        } else {
            "".to_string()
        };

        // Install apt packages
        let apt_pkgs = phase.apt_pkgs.clone().unwrap_or_default();
        let apt_pkgs_str = if apt_pkgs.is_empty() {
            "".to_string()
        } else {
            format!(
                "RUN apt-get update && apt-get install -y {}",
                apt_pkgs.join(" ")
            )
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

        let dockerfile_stmts = vec![
            build_path,
            run_path,
            install_nix_pkgs_str,
            apt_pkgs_str,
            phase_copy_cmd,
            cmds_str,
        ]
        .into_iter()
        .filter(|stmt| !stmt.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

        let dockerfile = formatdoc! {"
            # {name} phase
            {dockerfile_stmts}
        ", 
          name=phase.name,
          dockerfile_stmts=dockerfile_stmts
        };

        Ok(dockerfile)
    }

    fn write_supporting_files(
        &self,
        _options: &DockerBuilderOptions,
        _env: &Environment,
        output: &OutputDir,
    ) -> Result<()> {
        if self.uses_nix() {
            // Write the Nix expressions to the output directory
            let nix_file_name = format!("{}.nix", self.name);
            let nix_path = output.get_absolute_path(nix_file_name);
            let nix_expression = nix::create_nix_expression(self);

            let mut nix_file = File::create(nix_path).context("Creating Nix environment file")?;
            nix_file
                .write_all(nix_expression.as_bytes())
                .context("Unable to write Nix expression")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_generation() {
        let mut phase = Phase::new("test");
        phase.add_cmd("echo test");
        phase.add_apt_pkgs(vec!["wget".to_owned()]);

        let dockerfile = phase
            .generate_dockerfile(
                &DockerBuilderOptions::default(),
                &Environment::default(),
                &OutputDir::default(),
            )
            .unwrap();

        assert!(dockerfile.contains("echo test"));
        assert!(dockerfile.contains("apt-get update"));
        assert!(dockerfile.contains("wget"));
    }

    #[test]
    fn test_plan_generation() {
        let mut plan = BuildPlan::default();

        let mut test1 = Phase::new("test1");
        test1.add_cmd("echo test1");
        plan.add_phase(test1);

        let mut test2 = Phase::new("test2");
        test2.add_cmd("echo test2");
        plan.add_phase(test2);

        let dockerfile = plan
            .generate_dockerfile(
                &DockerBuilderOptions::default(),
                &Environment::default(),
                &OutputDir::default(),
            )
            .unwrap();

        assert!(dockerfile.contains("echo test1"));
        assert!(dockerfile.contains("echo test2"));
    }
}
