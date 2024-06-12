use uuid::Uuid;
use std::str;
use crate::nixpacks::builder::docker::docker_buildx_builder::DockerBuildxBuilder;

pub struct DockerBuildxBuilderManager {
    network_name: String,
    builder_name: String,
    buildx_builder: DockerBuildxBuilder,
    builder_name_before_creation: Option<String>,
}

impl DockerBuildxBuilderManager {
    pub fn create_manager_for_network(network: String) -> DockerBuildxBuilderManager {
        let builder_name = format!("nixpacks-builder-{}", Uuid::new_v4());

        DockerBuildxBuilderManager { builder_name_before_creation: None, builder_name, network_name: network, buildx_builder: DockerBuildxBuilder::default() }
    }

    pub fn validate_network_exists(&self) -> Result<bool, &str> {
        let network_exists = self.buildx_builder.check_if_network_exists(&self.network_name);
        if network_exists.is_err() {
            return Err("Could not check if network exists, cannot continue");
        }

        return Ok(true);
    }

    // Creates build and switches to builder instance
    pub fn create(&mut self) -> Result<bool, &str> {
        let builder_created = self.buildx_builder.create_buildx_builder(&self.builder_name, &self.network_name);

        if builder_created.is_err() {
            return Err("Could not create Docker builder, cannot continue");
        }

        let current_builder_result = self.buildx_builder.get_builder_name();
        if (current_builder_result.is_err()) {
            return Err("Could not fetch current Docker builder name, cannot continue");
        }

        let current_builder_name = current_builder_result.unwrap();

        // store current_builder_name so the finish function can switch to that builder
        self.builder_name_before_creation = Some(current_builder_name);


        return Ok(true);
    }

    pub fn finish(&self) -> Result<bool, &str> {
        if self.builder_name_before_creation.is_none() {
            return Err("Builder before creation is not set - there is nothing to revert to. Cannot continue");
        }

        // fetch currrent_builder_name as std::str
        let previous_builder = self.builder_name_before_creation.as_ref().unwrap();

        let builder_set_active_result = self.buildx_builder.set_buildx_builder_active(previous_builder);
        if(builder_set_active_result.is_err()) {
            return Err("Could not switch to previous Docker builder, cannot continue");
        }

        let removal_buildx_builder_result = self.buildx_builder.remove_buildx_builder(&self.builder_name);
        if removal_buildx_builder_result.is_err() {
            return Err("Could not remove Docker builder");
        }


        return Ok(true);
    }
}