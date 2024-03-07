use async_graphql::SimpleObject;
use object_store::path::Path;

pub mod file_hasher;
pub mod file_reader;
pub mod ipfs;
pub mod manifest_fetcher;
pub mod remote_object_store;
pub mod store;
// pub mod subfile_reader;

use serde::{Deserialize, Serialize};

use crate::{
    errors::Error,
    manifest::{file_hasher::verify_chunk, ipfs::is_valid_ipfs_hash},
};

/* Public Manifests */

/// Better mapping of files and file manifests
#[derive(Serialize, Deserialize, Clone, Debug, SimpleObject)]
pub struct BundleManifest {
    pub files: Vec<FileMetaInfo>,
    pub file_type: String,
    pub spec_version: String,
    pub description: String,
    pub chain_id: String,
    pub block_range: BlockRange,
    // pub identifier: String,
    // pub publisher_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, SimpleObject)]
// #[graphql(input_name = "MyObjInput")] // Note: You must use the input_name attribute to define a new name for the input type, otherwise a runtime error will occur.
pub struct FileMetaInfo {
    pub name: String,
    pub hash: String,
    // Some tags for discovery and categorization
    // pub block_range: BlockRange,
}

/* File manifest */
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, SimpleObject)]
pub struct FileManifest {
    pub total_bytes: u64,
    pub chunk_size: u64,
    pub chunk_hashes: Vec<String>,
}

// impl FileManifest {
//     pub fn new(read_dir: &str, file_name: &str, chunk_size: u64) -> Result<FileManifest, Error> {
//         let file_path = format_path(read_dir, file_name);
//         // let merkle_root = hex::encode(merkle_tree.root());
//         // let chunk_hashes: Vec<String> = merkle_tree.nodes().iter().map(hex::encode).collect();
//         // let (total_bytes, chunks) = chunk_file(Path::new(&file_path), chunk_size)?;
//         let chunk_hashes: Vec<String> = chunks.iter().map(|c| hash_chunk(c)).collect();

//         Ok(FileManifest {
//             total_bytes,
//             chunk_size,
//             chunk_hashes,
//         })
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, SimpleObject)]
pub struct FileManifestMeta {
    pub meta_info: FileMetaInfo,
    pub file_manifest: FileManifest,
}

/* Bundle - packaging of file manifests mapped into local files */
//TODO: Add GraphQL derivation
#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct Bundle {
    pub ipfs_hash: String,
    // #[graphql(skip)] // require admin for this field
    // pub local_path: PathBuf,
    pub manifest: BundleManifest,
    /// IPFS hash, File manifest spec
    pub file_manifests: Vec<FileManifestMeta>,
}

#[derive(Clone, Debug)]
pub struct LocalBundle {
    pub bundle: Bundle,
    pub local_path: object_store::path::Path,
}

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
pub struct BlockRange {
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
}

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

/// Validate the bundle configurations at initialization
pub fn validate_bundle_entries(entries: Vec<String>) -> Result<Vec<(String, Path)>, Error> {
    let mut results = Vec::new();

    for entry in entries {
        results.push(validate_bundle_entry(entry)?);
    }

    Ok(results)
}

/// Bundle entry must be in the format of "valid_ipfs_hash:valid_local_path"
pub fn validate_bundle_entry(entry: String) -> Result<(String, Path), Error> {
    let parts: Vec<&str> = entry.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::InvalidConfig(format!(
            "Invalid format for entry: {}",
            entry
        )));
    }

    let ipfs_hash = parts[0];
    let local_path = parts[1];
    validate_bundle_and_location(ipfs_hash, local_path)
}

// Check for valid ipfs hash and path
pub fn validate_bundle_and_location(
    ipfs_hash: &str,
    local_path: &str,
) -> Result<(String, Path), Error> {
    if !is_valid_ipfs_hash(ipfs_hash) {
        return Err(Error::InvalidConfig(format!(
            "Invalid IPFS hash: {}",
            ipfs_hash
        )));
    }

    // Validate local path

    Ok((ipfs_hash.to_string(), Path::from(local_path)))
}
