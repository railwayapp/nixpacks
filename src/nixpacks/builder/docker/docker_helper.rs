use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::str;

#[derive(Serialize, Deserialize, Debug)]
struct ContainerInfoFromDocker {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "IPv4Address")]
    ipv4_address: String,
}

pub struct ContainerInfo {
    pub name: String,
    pub ipv4_address: String,
    pub ipv4_address_without_mask: String,
}

type Containers = HashMap<String, ContainerInfo>;

pub struct DockerHelper {}

impl DockerHelper {
    pub fn containers_in_network(network: &str) -> Result<Containers, Box<dyn Error>> {
        let output = Command::new("docker")
            .arg("network")
            .arg("inspect")
            .arg(network)
            .arg("-f")
            .arg("{{json .Containers}}")
            .output()?;

        if output.status.success() {
            let containers_string = str::from_utf8(&output.stdout)?;
            let containers: HashMap<String, ContainerInfoFromDocker> =
                serde_json::from_str(containers_string)?;

            let mut vec = Vec::new();
            for info in containers.values() {
                let ipv4 = info.ipv4_address.split('/').next().unwrap();
                let container_info = ContainerInfo {
                    name: info.name.clone(),
                    ipv4_address: info.ipv4_address.clone(),
                    ipv4_address_without_mask: ipv4.to_string(),
                };
                vec.push(container_info);
            }

            return Ok(vec
                .into_iter()
                .map(|info| (info.name.clone(), info))
                .collect());
        }

        Err("Docker command failed".into())
    }
}
