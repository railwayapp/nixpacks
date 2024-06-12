use std::{
    process::Command,
    str,
};
use std::process::Output;
use uuid::Uuid;

#[derive(Default)]
pub struct DockerBuildxBuilder {

}

impl DockerBuildxBuilder {
    pub fn get_builder_name(&self) -> Result<String, std::io::Error> {
        let output: Output = Command::new("sh")
            .arg("-c")
            .arg("docker buildx inspect | grep -m 1 'Name:' | awk '{print $2}'")
            .output()?;

        if output.status.success() {
            let builder_name = str::from_utf8(&output.stdout).unwrap().trim().to_string();
            Ok(builder_name)
        } else {
            let stdout = str::from_utf8(&output.stdout).unwrap();
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get builder name",
            ))
        }
    }

    pub fn check_if_network_exists(&self, network_name: &str) -> Result<bool, std::io::Error> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("docker network inspect {}", network_name))
            .output()?;

        if output.status.success() {
            Ok(true)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to check if network exists",
            ))
        }
    }

    pub fn create_buildx_builder(&self, builder_name: &str, network_name: &str) -> Result<bool, std::io::Error> {

        let network_exists = self.check_if_network_exists(network_name);
        if network_exists.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to check if network exists",
            ));
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("docker buildx create --name {} --driver docker-container --driver-opt network={}", builder_name, network_name))
            .output()?;

        if output.status.success() {
            Ok(true)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create builder",
            ))
        }
    }

    pub fn set_buildx_builder_active(&self, builder_name: &str) -> Result<bool, std::io::Error> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("docker buildx use {}", builder_name))
            .output()?;

        if output.status.success() {
            Ok(true)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to switch builder",
            ))
        }
    }

    pub fn remove_buildx_builder(&self, builder_name: &str) -> Result<bool, std::io::Error> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("docker buildx rm {}", builder_name))
            .output()?;

        if output.status.success() {
            Ok(true)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to remove builder",
            ))
        }
    }

    pub fn create_docker_network(&self, network_name: &str) -> Result<bool, std::io::Error> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("docker network create {}", network_name))
            .output()?;

        if output.status.success() {
            Ok(true)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create network",
            ))
        }
    }

    pub fn docker_network_exists(&self, network_name: &str) -> Result<bool, std::io::Error> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("docker network inspect {}", network_name))
            .output()?;

        if output.status.success() {
            Ok(true)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to check if network exists",
            ))
        }
    }
}