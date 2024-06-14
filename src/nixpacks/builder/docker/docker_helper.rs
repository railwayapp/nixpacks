use std::collections::HashMap;
use std::error::Error;
use std::process::Command;
use std::str;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ContainerInfoFromDocker {
    Name: String,
    EndpointID: String,
    MacAddress: String,
    IPv4Address: String,
    IPv6Address: String,
}

pub struct ContainerInfo {
    pub Name: String,
    pub EndpointID: String,
    pub MacAddress: String,
    pub IPv4Address: String,
    pub IPv6Address: String,
    pub IPv4WithoutMask: String,
}

type Containers = HashMap<String, ContainerInfo>;


pub struct DockerHelper {

}

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
            let containers: HashMap<String, ContainerInfoFromDocker> = serde_json::from_str(containers_string)?;

            let mut vec = Vec::new();
            for (name, info) in containers.iter() {
                let ipv4 = info.IPv4Address.split('/').next().unwrap();
                let container_info = ContainerInfo {
                    Name: info.Name.clone(),
                    EndpointID: info.EndpointID.clone(),
                    MacAddress: info.MacAddress.clone(),
                    IPv4Address: info.IPv4Address.clone(),
                    IPv6Address: info.IPv6Address.clone(),
                    IPv4WithoutMask: ipv4.to_string(),
                };
                vec.push(container_info);
            }

            return  Ok(vec.into_iter().map(|info| (info.Name.clone(), info)).collect());
        }
        let err = str::from_utf8(&output.stderr)?;
        eprintln!("Docker command failed: {}", err);

        return Err("Docker command failed".into());
    }
}