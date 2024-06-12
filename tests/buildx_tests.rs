use uuid::Uuid;
use nixpacks::nixpacks::builder::docker::docker_buildx_builder::DockerBuildxBuilder;


#[tokio::test]
async fn test_validate_network_gets_created() {
    let builder = DockerBuildxBuilder::default();

    let test_network_name = format!("nixpacks-test-network-{}", Uuid::new_v4());

    let network_exists = builder.check_if_network_exists(test_network_name.as_str());
    assert_eq!(network_exists.is_ok(), false);
    let network_created_result = builder.create_docker_network(test_network_name.as_str());

    assert_eq!(network_created_result.is_ok(), true);

    let network_exists = builder.check_if_network_exists(test_network_name.as_str());
    assert_eq!(network_exists.is_ok(), true);
}

#[tokio::test]
async fn test_validate_cannot_create_builder_without_valid_network() {
    let builder = DockerBuildxBuilder::default();

    let test_network_name = format!("nixpacks-test-network-{}", Uuid::new_v4());
    let test_builder_name = format!("nixpacks-test-builder-{}", Uuid::new_v4());
    let network_exists = builder.check_if_network_exists(test_network_name.as_str());
    assert_eq!(network_exists.is_ok(), false);

    let builder_result = builder.create_buildx_builder(test_builder_name.as_str(), test_network_name.as_str());
    assert_eq!(builder_result.is_ok(), false);
}

#[tokio::test]
async fn test_should_always_be_able_to_fetch_builder_name() {
    let builder = DockerBuildxBuilder::default();

    let builder_name = builder.get_builder_name();
    assert_eq!(builder_name.unwrap().is_empty(), false);
}