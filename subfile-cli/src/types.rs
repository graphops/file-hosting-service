use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{file_hasher::ChunkFile, publisher::SubfileManifest};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subfile {
    pub ipfs_hash: String,
    pub local_path: PathBuf,
    pub manifest: SubfileManifest,
    pub chunk_files: Vec<ChunkFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockRange {
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct SeedCreationArg {
//     pub file_name: String,
//     pub file_path: Option<String>,
//     pub file_link: Option<String>,
//     pub file_type: String,
//     pub version: String,
//     pub identifier: String,
//     pub trackers: Vec<String>,
//     pub block_range: BlockRange,
// }

// impl SeedCreationArg {
//     pub fn build(
//         file_name: String,
//         file_type: String,
//         file_path: Option<String>,
//         file_link: Option<String>,
//         version: String,
//         identifier: String,
//         trackers: Vec<String>,
//         start_block: Option<u64>,
//         end_block: Option<u64>,
//     ) -> Self {
//         Self {
//             file_name,
//             file_type,
//             file_path,
//             file_link,
//             version,
//             identifier,
//             trackers,
//             block_range: BlockRange {
//                 start_block,
//                 end_block,
//             },
//         }
//     }

//     pub fn subfile(&self) -> Result<Subfile, anyhow::Error> {
//         // Placeholder: Replace with actual logic to generate magnet link
//         let file_link = if let Some(link) = self.file_link.clone() {
//             link
//         } else {
//             String::from("temp")
//         };

//         Ok(Subfile {
//             file_link: file_link.to_string(),
//             file_name: self.file_name.clone(),
//             file_type: self.file_type.clone(),
//             version: self.version.clone(),
//             identifier: self.identifier.clone(),
//             trackers: self.trackers.clone(),
//             block_range: self.block_range.clone(),
//         })
//     }
// }

// fn convert_to_subfile(value: serde_yaml::Value) -> Result<Subfile, anyhow::Error> {
//     let subfile: Subfile = serde_yaml::from_value(value)?;
//     Ok(subfile)
// }

// #[derive(ValueEnum, Clone, Debug, Serialize, Deserialize, Default)]
// pub enum FileType {
//     #[default]
//     SqlSnapshot,
//     Flatfiles,
// }

// impl FromStr for FileType {
//     type Err = &'static str;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "sql_snapshot" => Ok(FileType::SqlSnapshot),
//             "flatfiles" => Ok(FileType::Flatfiles),
//             _ => Err("Invalid file type"),
//         }
//     }
// }
