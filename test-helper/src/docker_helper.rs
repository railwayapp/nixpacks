use std::collections::HashMap;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
struct ContainerInfo {
    Name: String,
    EndpointID: String,
    MacAddress: String,
    IPv4Address: String,
    IPv6Address: String,
}
type Containers = HashMap<String, ContainerInfo>;

#[proc_macro]
pub fn fetch_docker_containers_in_network(network: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("docker")
        .arg("network")
        .arg("inspect")
        .arg(network)
        .arg("-f")
        .arg("'{{json .Containers}}'")
        .output()?;

    if output.status.success() {
        let containers = str::from_utf8(&output.stdout)?;
        let v: Containers = serde_json::from_str(containers)?;
        println!("{}", containers);
    } else {
        let err = str::from_utf8(&output.stderr)?;
        eprintln!("Docker command failed: {}", err);
    }

    Ok(output.status.success())
}