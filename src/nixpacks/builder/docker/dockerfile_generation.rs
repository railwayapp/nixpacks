use super::{
    file_server::FileServerConfig, incremental_cache::IncrementalCache, utils, DockerBuilderOptions,
};
use crate::nixpacks::{
    app,
    environment::Environment,
    images::DEFAULT_BASE_IMAGE,
    nix::{create_nix_expressions_for_phases, nix_file_names_for_phases, setup_files_for_phases},
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
        _file_server_config: Option<FileServerConfig>,
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
        file_server_config: Option<FileServerConfig>,
    ) -> Result<String> {
        let plan = self;

        let setup_files = setup_files_for_phases(&plan.phases.clone().unwrap_or_default());
        let setup_copy_cmds = utils::get_copy_commands(&setup_files, APP_DIR).join("\n");

        let nix_file_names = nix_file_names_for_phases(&plan.phases.clone().unwrap_or_default());

        let mut nix_install_cmds: Vec<String> = Vec::new();
        for name in nix_file_names {
            let nix_file = output.get_relative_path(name);

            let nix_file_path = nix_file
                .to_slash()
                .context("Failed to convert nix file path to slash path.")?;

            nix_install_cmds.push(format!(
                "COPY {nix_file_path} {nix_file_path}\nRUN nix-env -if {nix_file_path} && nix-collect-garbage -d"
            ));
        }
        let nix_install_cmds = nix_install_cmds.join("\n");

        let apt_pkgs = self.all_apt_packages();
        let apt_pkgs_str = if apt_pkgs.is_empty() {
            String::new()
        } else {
            format!(
                "RUN apt-get update && apt-get install -y --no-install-recommends {}",
                apt_pkgs.join(" ")
            )
        };

        let variables = plan.variables.clone().unwrap_or_default();
        let args_string = if variables.is_empty() {
            String::new()
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
            String::new()
        } else {
            let rel_assets_path = output.get_relative_path("assets");
            let rel_assets_slash_path = rel_assets_path
                .to_slash()
                .context("Failed to convert nix file path to slash path.")?;
            format!("COPY {rel_assets_slash_path} {}", app::ASSETS_DIR)
        };

        let phases = plan.get_sorted_phases()?;

        let dockerfile_phases = phases
            .into_iter()
            .map(|phase| {
                let phase_dockerfile = phase
                    .generate_dockerfile(options, env, output, file_server_config.clone())
                    .context(format!(
                        "Generating Dockerfile for phase {}",
                        phase.get_name()
                    ))?;

                Ok(phase_dockerfile)
            })
            .collect::<Result<Vec<_>>>()?;
        let dockerfile_phases_str = dockerfile_phases.join("\n");

        let start_phase_str = plan
            .start_phase
            .clone()
            .unwrap_or_default()
            .generate_dockerfile(options, env, output, file_server_config)?;

        let base_image = plan
            .build_image
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_IMAGE.to_string());

        let dockerfile = formatdoc! {"
            FROM {base_image}

            WORKDIR {APP_DIR}

            {setup_copy_cmds}
            {nix_install_cmds}
            {apt_pkgs_str}
            {assets_copy_cmd}
            {args_string}

            {dockerfile_phases_str}

            {start_phase_str}
        ", 
        base_image=base_image,
        APP_DIR=APP_DIR,
        setup_copy_cmds=setup_copy_cmds,
        nix_install_cmds=nix_install_cmds,
        apt_pkgs_str=apt_pkgs_str,
        assets_copy_cmd=assets_copy_cmd,
        args_string=args_string,
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

        let nix_expressions =
            create_nix_expressions_for_phases(&self.phases.clone().unwrap_or_default());

        for (name, nix_expression) in nix_expressions {
            let nix_path = output.get_absolute_path(name);
            let mut nix_file = File::create(nix_path).context("Creating Nix environment file")?;
            nix_file
                .write_all(nix_expression.as_bytes())
                .context("Unable to write Nix expression")?;
        }

        for phase in self.get_sorted_phases()? {
            phase
                .write_supporting_files(options, env, output)
                .context(format!("Writing files for phase {}", phase.get_name()))?;
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
                        .context(format!("Creating parent directory for {name}"))?;
                    let mut file =
                        File::create(path).context(format!("Creating asset file for {name}"))?;
                    file.write_all(content.as_bytes())
                        .context(format!("Writing asset {name}"))?;
                }
            }
        }

        Ok(())
    }

    fn all_apt_packages(&self) -> Vec<String> {
        self.phases
            .clone()
            .unwrap_or_default()
            .values()
            .flat_map(|phase| phase.apt_pkgs.clone().unwrap_or_default())
            .collect()
    }
}

impl DockerfileGenerator for StartPhase {
    fn generate_dockerfile(
        &self,
        _options: &DockerBuilderOptions,
        _env: &Environment,
        _output: &OutputDir,
        _file_server_config: Option<FileServerConfig>,
    ) -> Result<String> {
        let start_cmd = match &self.cmd {
            Some(cmd) => utils::get_exec_command(cmd),
            None => String::new(),
        };

        let dockerfile: String = match &self.run_image {
            Some(run_image) => {
                let copy_cmds = utils::get_copy_from_commands(
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
                  {copy_cmds}
                  {start_cmd}
                ",
                run_image=run_image,
                APP_DIR=APP_DIR,
                copy_cmds=copy_cmds.join("\n"),
                start_cmd=start_cmd,}
            }
            None => {
                formatdoc! {"
                  # start
                  COPY . /app
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
        _output: &OutputDir,
        file_server_config: Option<FileServerConfig>,
    ) -> Result<String> {
        if !self.runs_docker_commands() {
            return Ok(format!("# {} phase\n# noop\n", self.get_name()));
        }

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
                format!("ENV PATH {joined_paths}:$PATH"),
                format!("RUN printf '\\nPATH={joined_paths}:$PATH' >> /root/.profile"),
            )
        } else {
            (String::new(), String::new())
        };

        // Copy over app files
        let phase_files = match (phase.get_name().as_str(), &phase.only_include_files) {
            (_, Some(files)) => files.clone(),
            _ => vec![".".to_string()],
        };
        let phase_copy_cmds = utils::get_copy_commands(&phase_files, APP_DIR);

        let cache_mount = utils::get_cache_mount(&cache_key, &phase.cache_directories);
        let cmds_str = if options.incremental_cache_image.is_some() {
            let image = &options.incremental_cache_image.clone().unwrap();
            let cache_copy_in_command = if IncrementalCache::is_image_exists(image)? {
                IncrementalCache::get_copy_to_image_command(&phase.cache_directories, image)
                    .join("\n")
            } else {
                String::new()
            };

            let cache_copy_out_command = IncrementalCache::get_copy_from_image_command(
                &phase.cache_directories,
                file_server_config,
            );

            let run_commands = [
                phase.cmds.clone().unwrap_or_default(),
                cache_copy_out_command,
            ]
            .concat()
            .iter()
            .map(|s| format!("RUN {s}"))
            .collect::<Vec<_>>()
            .join("\n");

            format!("{cache_copy_in_command}\n{run_commands}")
        } else {
            phase
                .cmds
                .clone()
                .unwrap_or_default()
                .iter()
                .map(|s| format!("RUN {cache_mount} {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let dockerfile_stmts = vec![build_path, run_path, phase_copy_cmds.join("\n"), cmds_str]
            .into_iter()
            .filter(|stmt| !stmt.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        let dockerfile = formatdoc! {"
            # {name} phase
            {dockerfile_stmts}
        ", 
          name=phase.get_name(),
          dockerfile_stmts=dockerfile_stmts
        };

        Ok(dockerfile)
    }

    fn write_supporting_files(
        &self,
        _options: &DockerBuilderOptions,
        _env: &Environment,
        _output: &OutputDir,
    ) -> Result<()> {
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
                Some(FileServerConfig::default()),
            )
            .unwrap();

        assert!(dockerfile.contains("echo test"));
    }

    #[test]
    fn test_plan_generation() {
        let mut plan = BuildPlan::default();

        let mut test1 = Phase::new("test1");
        test1.add_cmd("echo test1");
        test1.add_apt_pkgs(vec!["wget".to_owned()]);
        plan.add_phase(test1);

        let mut test2 = Phase::new("test2");
        test2.add_cmd("echo test2");
        plan.add_phase(test2);

        let dockerfile = plan
            .generate_dockerfile(
                &DockerBuilderOptions::default(),
                &Environment::default(),
                &OutputDir::default(),
                Some(FileServerConfig::default()),
            )
            .unwrap();

        assert!(dockerfile.contains("echo test1"));
        assert!(dockerfile.contains("echo test2"));
        assert!(dockerfile.contains("apt-get update"));
        assert!(dockerfile.contains("wget"));
    }
}
