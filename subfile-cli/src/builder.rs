use std::fs::File;
use std::io::Write;

use crate::{
    config::Builder,
    ipfs::{AddResponse, IpfsClient},
    types::{SeedCreationArg, Subfile},
};

pub async fn seed(client: &IpfsClient, config: &Builder) -> Result<AddResponse, anyhow::Error> {
    // TODO: use a library or external tool to create a magnet link. (intermodal)
    let subfile_args = SeedCreationArg::build(
        config.file_path.clone().unwrap_or_default(),
        config.file_type.clone(),
        config.file_path.clone(),
        config.file_link.clone(),
        config.file_version.clone(),
        config.identifier.clone(),
        config.trackers.clone(),
        config.start_block,
        config.end_block,
    );
    let subfile: Subfile = subfile_args.subfile()?;

    // Convert the Subfile struct into a `subfile.yaml` file.
    let yaml_str = serde_yaml::to_string(&subfile)?;
    let mut file = File::create(&config.yaml_store)?;
    file.write_all(yaml_str.as_bytes())?;

    // Add `subfile.yaml` to IPFS.
    let added: AddResponse = client.add(yaml_str.as_bytes().to_vec()).await?;
    tracing::info!(
        added = tracing::field::debug(&added),
        client = tracing::field::debug(&client),
        "Added yaml file to IPFS"
    );

    Ok(added)
}

pub async fn create_subfile(client: &IpfsClient, config: &Builder) -> Result<AddResponse, anyhow::Error> {
    // TODO: use a library or external tool to create a magnet link. (intermodal)
    let subfile_args = SeedCreationArg::build(
        config.file_path.clone().unwrap_or_default(),
        config.file_type.clone(),
        config.file_path.clone(),
        config.file_link.clone(),
        config.file_version.clone(),
        config.identifier.clone(),
        config.trackers.clone(),
        config.start_block,
        config.end_block,
    );
    let subfile: Subfile = subfile_args.subfile()?;

    // Convert the Subfile struct into a `subfile.yaml` file.
    let yaml_str = serde_yaml::to_string(&subfile)?;
    let mut file = File::create(&config.yaml_store)?;
    file.write_all(yaml_str.as_bytes())?;

    // Add `subfile.yaml` to IPFS.
    let added: AddResponse = client.add(yaml_str.as_bytes().to_vec()).await?;
    tracing::info!(
        added = tracing::field::debug(&added),
        client = tracing::field::debug(&client),
        "Added yaml file to IPFS"
    );

    Ok(added)
}
