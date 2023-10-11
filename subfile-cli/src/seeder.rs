use std::fs::File;
use std::io::Write;

use crate::{
    config::Seeder,
    ipfs::{AddResponse, IpfsClient},
    types::{SeedCreationArg, Subfile},
};

// pub fn build_and_write(){
//     let subfile = Subfile {
//         magnet_link: "magnet:?xt=urn:btih:example".to_string(),
//         file_type: "sql_snapshot".to_string(),
//         identifier: "subgraph_deployment_hash".to_string(),
//         block_range: BlockRange {
//             start_block: Some(10),
//             end_block: Some(100),
//         },
//     };

//     match write_subfile_to_yaml(&subfile, "output.yaml") {
//         Ok(_) => println!("Successfully wrote to output.yaml"),
//         Err(e) => eprintln!("Error: {}", e),
//     }
// }

pub async fn seed(client: &IpfsClient, config: &Seeder) -> Result<AddResponse, anyhow::Error> {
    // TODO: use a library or external tool to create a magnet link. (intermodal)
    let subfile_args = SeedCreationArg::build(
        config.file_path.clone(),
        config.file_type.clone(),
        config.file_version.clone(),
        config.identifier.clone(),
        config.trackers.clone(),
        config.start_block,
        config.end_block,
    );
    let subfile: Subfile = subfile_args.into();

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
