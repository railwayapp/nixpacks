use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use super::Builder;
use crate::nixpacks::{app, files, logger::Logger, nix, plan::BuildPlan, NIX_PACKS_VERSION};
use anyhow::{bail, Context, Ok, Result};
use indoc::formatdoc;
use tempdir::TempDir;
use uuid::Uuid;

#[derive(Clone, Default, Debug)]
pub struct DockerBuilderOptions {
    pub name: Option<String>,
    pub out_dir: Option<String>,
    pub tags: Vec<String>,
    pub labels: Vec<String>,
    pub quiet: bool,
    pub force_buildkit: bool,
}

pub struct DockerBuilder {
    logger: Logger,
    options: DockerBuilderOptions,
}

impl Builder for DockerBuilder {
    fn create_image(&self, app_src: &str, plan: &BuildPlan) -> Result<()> {
        self.logger
            .log_section(format!("Building (nixpacks v{})", NIX_PACKS_VERSION).as_str());

        println!("{}", plan.get_build_string());

        let id = Uuid::new_v4();

        let dir = match &self.options.out_dir {
            Some(dir) => dir.into(),
            None => {
                let tmp = TempDir::new("nixpacks").context("Creating a temp directory")?;
                tmp.into_path()
            }
        };
        let dest = dir.to_str().context("Invalid temp directory path")?;
        let name = self.options.name.clone().unwrap_or_else(|| id.to_string());

        // Write everything to destination
        self.write_app(app_src, dest).context("Writing app")?;
        self.write_assets(plan, dest).context("Writing assets")?;
        self.write_dockerfile(plan, dest)
            .context("Writing Dockerfile")?;
        self.write_nix_expression(plan, dest)
            .context("Writing NIx expression")?;

        // Only build if the --out flag was not specified
        if self.options.out_dir.is_none() {
            let mut docker_build_cmd = self.get_docker_build_cmd(plan, name.as_str(), dest)?;

            // Execute docker build
            let build_result = docker_build_cmd.spawn()?.wait().context("Building image")?;

            if !build_result.success() {
                bail!("Docker build failed")
            }

            self.logger.log_section("Successfully Built!");

            println!("\nRun:");
            println!("  docker run -it {}", name);
        } else {
            println!("\nSaved output to:");
            println!("  {}", dest);
        }

        Ok(())
    }
}

impl DockerBuilder {
    pub fn new(logger: Logger, options: DockerBuilderOptions) -> DockerBuilder {
        DockerBuilder { logger, options }
    }

    fn get_docker_build_cmd(&self, plan: &BuildPlan, name: &str, dest: &str) -> Result<Command> {
        let mut docker_build_cmd = Command::new("docker");

        if docker_build_cmd.output().is_err() {
            bail!("Please install Docker to build the app https://docs.docker.com/engine/install/")
        }

        if self.options.force_buildkit {
            docker_build_cmd.env("DOCKER_BUILDKIT", "1");
        }
        docker_build_cmd.arg("build").arg(dest).arg("-t").arg(name);

        if self.options.quiet {
            docker_build_cmd.arg("--quiet");
        }

        // Add build environment variables
        for (name, value) in plan.variables.clone().unwrap_or_default().iter() {
            docker_build_cmd
                .arg("--build-arg")
                .arg(format!("{}={}", name, value));
        }

        // Add user defined tags and labels to the image
        for t in self.options.tags.clone() {
            docker_build_cmd.arg("-t").arg(t);
        }
        for l in self.options.labels.clone() {
            docker_build_cmd.arg("--label").arg(l);
        }

        Ok(docker_build_cmd)
    }

    fn write_app(&self, app_src: &str, dest: &str) -> Result<()> {
        files::recursive_copy_dir(app_src, &dest)
    }

    fn write_dockerfile(&self, plan: &BuildPlan, dest: &str) -> Result<()> {
        let dockerfile = self.create_dockerfile(plan);

        let dockerfile_path = PathBuf::from(dest).join(PathBuf::from("Dockerfile"));
        File::create(dockerfile_path.clone()).context("Creating Dockerfile file")?;
        fs::write(dockerfile_path, dockerfile).context("Writing Dockerfile")?;

        Ok(())
    }

    fn write_nix_expression(&self, plan: &BuildPlan, dest: &str) -> Result<()> {
        let nix_expression = nix::create_nix_expression(plan);

        let nix_path = PathBuf::from(dest).join(PathBuf::from("environment.nix"));
        let mut nix_file = File::create(nix_path).context("Creating Nix environment file")?;
        nix_file
            .write_all(nix_expression.as_bytes())
            .context("Unable to write Nix expression")?;

        Ok(())
    }

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

    fn create_dockerfile(&self, plan: &BuildPlan) -> String {
        let app_dir = "/app/";
        let assets_dir = app::ASSETS_DIR;

        let setup_phase = plan.setup.clone().unwrap_or_default();
        let install_phase = plan.install.clone().unwrap_or_default();
        let build_phase = plan.build.clone().unwrap_or_default();
        let start_phase = plan.start.clone().unwrap_or_default();
        let variables = plan.variables.clone().unwrap_or_default();
        let static_assets = plan.static_assets.clone().unwrap_or_default();

        // -- Variables
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

        // -- Setup
        let mut setup_files: Vec<String> = vec!["environment.nix".to_string()];
        if let Some(mut setup_file_deps) = setup_phase.only_include_files {
            setup_files.append(&mut setup_file_deps);
        }
        let setup_copy_cmd = format!("COPY {} {}", setup_files.join(" "), app_dir);

        let mut apt_get_cmd = "".to_string();
        // using apt will break build reproducibility
        if !setup_phase.apt_pkgs.clone().unwrap_or_default().is_empty() {
            let apt_pkgs = setup_phase.apt_pkgs.unwrap_or_default().join(" ");
            apt_get_cmd = format!("RUN apt-get update && apt-get install -y {}", apt_pkgs);
        }
        let setup_cmd = setup_phase
            .cmds
            .unwrap_or_default()
            .iter()
            .map(|c| format!("RUN {}", c))
            .collect::<Vec<String>>()
            .join("\n");

        // -- Static Assets
        let assets_copy_cmd = if !static_assets.is_empty() {
            static_assets
                .into_keys()
                .map(|name| format!("COPY assets/{} {}{}", name, assets_dir, name))
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            "".to_string()
        };

        // -- Install
        let install_cmd = install_phase
            .cmds
            .unwrap_or_default()
            .iter()
            .map(|c| format!("RUN {}", c))
            .collect::<Vec<String>>()
            .join("\n");

        let (build_path, run_path) = if let Some(paths) = install_phase.paths {
            let joined_paths = paths.join(":");
            (
                format!("ENV PATH {}:$PATH", joined_paths),
                format!("RUN printf '\\nPATH={joined_paths}:$PATH' >> /root/.profile"),
            )
        } else {
            ("".to_string(), "".to_string())
        };

        // Files to copy for install phase
        // If none specified, copy over the entire app
        let install_files = install_phase
            .only_include_files
            .clone()
            .unwrap_or_else(|| vec![".".to_string()]);

        // -- Build
        let build_cmd = build_phase
            .cmds
            .unwrap_or_default()
            .iter()
            .map(|c| format!("RUN {}", c))
            .collect::<Vec<String>>()
            .join("\n");

        let build_files = build_phase.only_include_files.unwrap_or_else(|| {
            // Only copy over the entire app if we haven't already in the install phase
            if install_phase.only_include_files.is_none() {
                Vec::new()
            } else {
                vec![".".to_string()]
            }
        });

        // -- Start
        let start_cmd = start_phase
            .cmd
            .map(|cmd| format!("CMD {}", cmd))
            .unwrap_or_else(|| "".to_string());

        // If we haven't yet copied over the entire app, do that before starting
        let start_files = start_phase.only_include_files.clone();

        let run_image_setup = match start_phase.run_image {
            Some(run_image) => {
                // RUN true to prevent a Docker bug https://github.com/moby/moby/issues/37965#issuecomment-426853382
                format! {"
                FROM {run_image}
                WORKDIR {app_dir}
                COPY --from=0 /etc/ssl/certs /etc/ssl/certs
                RUN true
                {copy_cmd}
            ",
                    run_image=run_image,
                    app_dir=app_dir,
                    copy_cmd=get_copy_from_command("0", &start_files.unwrap_or_default(), app_dir)
                }
            }
            None => get_copy_command(
                // If no files specified and no run image, copy everything in /app/ over
                &start_files.unwrap_or_else(|| vec![".".to_string()]),
                app_dir,
            ),
        };

        let dockerfile = formatdoc! {"
          FROM {base_image}

          WORKDIR {app_dir}

          # Setup
          {setup_copy_cmd}
          RUN nix-env -if environment.nix
          {apt_get_cmd}
          {setup_cmd}
          
          {assets_copy_cmd}

          # Load environment variables
          {args_string}

          # Install
          {install_copy_cmd}
          {install_cmd}

          {build_path}
          {run_path}

          # Build
          {build_copy_cmd}
          {build_cmd}

          # Start
          {run_image_setup}
          {start_cmd}
        ",
        base_image=setup_phase.base_image,
        install_copy_cmd=get_copy_command(&install_files, app_dir),
        build_copy_cmd=get_copy_command(&build_files, app_dir)};

        dockerfile
    }
}

fn get_copy_command(files: &[String], app_dir: &str) -> String {
    if files.is_empty() {
        "".to_owned()
    } else {
        format!("COPY {} {}", files.join(" "), app_dir)
    }
}

fn get_copy_from_command(from: &str, files: &[String], app_dir: &str) -> String {
    if files.is_empty() {
        format!("COPY --from=0 {} {}", app_dir, app_dir)
    } else {
        format!(
            "COPY --from={} {} {}",
            from,
            files
                .iter()
                .map(|f| f.replace("./", app_dir))
                .collect::<Vec<_>>()
                .join(" "),
            app_dir
        )
    }
}
