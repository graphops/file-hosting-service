use std::fs::File;
use std::io::Write;

use crate::{
    config::Seeder,
    ipfs::{AddResponse, IpfsClient},
    types::{BlockRange, Subfile},
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

pub async fn seed(client: &IpfsClient, seeder_config: &Seeder) -> Result<AddResponse, anyhow::Error> {
    // TODO: use a library or external tool to create a magnet link. (intermodal)
    let magnet_link = format!("magnet:?xt=urn:btih:{}", "some_hash_based_on_file");

    // Fill in a Subfile struct
    let subfile = Subfile {
        magnet_link,
        file_type: seeder_config.file_type.clone(),
        identifier: seeder_config.identifier.clone(),
        block_range: BlockRange {
            start_block: seeder_config.start_block,
            end_block: seeder_config.end_block,
        },
    };

    // Convert the Subfile struct into a `subfile.yaml` file.
    let yaml_str = serde_yaml::to_string(&subfile)?;
    let mut file = File::create(&seeder_config.yaml_store)?;
    file.write_all(yaml_str.as_bytes())?;

    // Add the `subgraph.yaml` to IPFS.
    let added: AddResponse = client.add(yaml_str.as_bytes().to_vec()).await?;
    println!("Added subgraph.yaml to IPFS with hash: {}", added.hash);

    Ok(added)
}
