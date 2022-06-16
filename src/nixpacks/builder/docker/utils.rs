use anyhow::{bail, Context, Ok, Result};

use std::process::Command;

pub fn get_sha256_of_image(name: &String) -> Result<String> {
    let output = Command::new("docker")
        .arg("images")
        .arg("--no-trunc")
        .arg("--quiet")
        .arg(name)
        .output()
        .context("failed to get sha256 hash of image")?;

    if output.status.success() {
        // TODO: Handle multiple lines of hashes

        match String::from_utf8_lossy(&output.stdout)
            .to_string()
            .strip_prefix("sha256:")
            .map(|s| s.to_string())
        {
            Some(hash) => Ok(hash),
            None => bail!("failed to parse Docker output"),
        }
    } else {
        // Failed to get sha256 of container
        bail!("failed to get sha256 hash of image")
    }
}

pub fn compare_hashes(cached_hash: &String, image_name: &String) -> Result<bool> {
    // TODO: Support comparing hash of non-local image

    let actual_image_hash = get_sha256_of_image(image_name)?;
    if actual_image_hash == *cached_hash {
        Ok(true)
    } else {
        Ok(false)
    }
}
