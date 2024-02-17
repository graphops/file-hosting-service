use async_graphql::SimpleObject;

pub mod file_hasher;
pub mod file_reader;
pub mod ipfs;
pub mod local_file_system;
pub mod manifest_fetcher;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{
    errors::Error,
    manifest::{
        file_hasher::{hash_chunk, verify_chunk},
        file_reader::{chunk_file, format_path, read_chunk},
        ipfs::is_valid_ipfs_hash,
    },
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

impl FileManifest {
    pub fn new(read_dir: &str, file_name: &str, chunk_size: u64) -> Result<FileManifest, Error> {
        let file_path = format_path(read_dir, file_name);
        // let merkle_root = hex::encode(merkle_tree.root());
        // let chunk_hashes: Vec<String> = merkle_tree.nodes().iter().map(hex::encode).collect();
        let (total_bytes, chunks) = chunk_file(Path::new(&file_path), chunk_size)?;

        let chunk_hashes: Vec<String> = chunks.iter().map(|c| hash_chunk(c)).collect();

        Ok(FileManifest {
            total_bytes,
            chunk_size,
            chunk_hashes,
        })
    }
}

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
    #[graphql(skip)] // require admin for this field
    pub local_path: PathBuf,
    pub manifest: BundleManifest,
    /// IPFS hash, File manifest spec
    pub file_manifests: Vec<FileManifestMeta>,
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

impl Bundle {
    /// Validate the local files against a given bundle specification
    pub fn validate_local_bundle(&self) -> Result<&Self, Error> {
        tracing::trace!(
            bundle = tracing::field::debug(self),
            "Read and verify bundle"
        );

        // Read all files in bundle to verify locally. This may cause a long initialization time
        for file_meta in &self.file_manifests {
            self.read_and_validate_file(file_meta)?;
        }

        tracing::trace!("Successfully verified the local serving files");
        Ok(self)
    }

    /// Read and validate file
    pub fn read_and_validate_file(&self, file: &FileManifestMeta) -> Result<(), Error> {
        // read file by file_manifest.file_name
        let meta_info = &file.meta_info;
        let file_manifest = &file.file_manifest;
        let mut file_path = self.local_path.clone();
        file_path.push(meta_info.name.clone());
        tracing::trace!(
            file_path = tracing::field::debug(&file_path),
            file_manifest = tracing::field::debug(&file_manifest),
            "Verify file"
        );

        // loop through file manifest  byte range
        for i in 0..(file_manifest.total_bytes / file_manifest.chunk_size + 1) {
            // read range
            let start = i * file_manifest.chunk_size;
            let end = u64::min(start + file_manifest.chunk_size, file_manifest.total_bytes) - 1;
            tracing::trace!(
                i,
                start_byte = tracing::field::debug(&start),
                end_byte = tracing::field::debug(&end),
                "Verify chunk index"
            );
            let chunk_hash = file_manifest.chunk_hashes[i as usize].clone();

            // read chunk
            let chunk_data = read_chunk(&file_path, (start, end))?;
            // verify chunk
            if !verify_chunk(&chunk_data, &chunk_hash) {
                tracing::error!(
                    file = tracing::field::debug(&file_path),
                    chunk_index = tracing::field::debug(&i),
                    chunk_hash = tracing::field::debug(&chunk_hash),
                    "Cannot locally verify the serving file"
                );
                return Err(Error::InvalidConfig(format!(
                    "Failed to validate the local version of file {}",
                    meta_info.hash
                )));
            }
        }
        Ok(())
    }
}

/// Validate the bundle configurations at initialization
pub fn validate_bundle_entries(entries: Vec<String>) -> Result<Vec<(String, PathBuf)>, Error> {
    let mut results = Vec::new();

    for entry in entries {
        results.push(validate_bundle_entry(entry)?);
    }

    Ok(results)
}

/// Bundle entry must be in the format of "valid_ipfs_hash:valid_local_path"
pub fn validate_bundle_entry(entry: String) -> Result<(String, PathBuf), Error> {
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
) -> Result<(String, PathBuf), Error> {
    if !is_valid_ipfs_hash(ipfs_hash) {
        return Err(Error::InvalidConfig(format!(
            "Invalid IPFS hash: {}",
            ipfs_hash
        )));
    }

    // Validate local path
    let path = PathBuf::from_str(local_path).map_err(|e| Error::InvalidConfig(e.to_string()))?;
    if !path.exists() {
        return Err(Error::InvalidConfig(format!(
            "Path do not exist: {}",
            local_path
        )));
    }

    Ok((ipfs_hash.to_string(), path))
}

#[cfg(test)]
mod tests {
    use crate::test_util::simple_bundle;

    #[test]
    fn test_read_and_validate_file() {
        let mut bundle = simple_bundle();
        let file_meta = bundle.file_manifests.first().unwrap();
        let result = bundle.read_and_validate_file(file_meta);
        assert!(result.is_ok());

        // Add tests for failure cases
        if let Some(file_meta) = bundle.file_manifests.first_mut() {
            if let Some(first_hash) = file_meta.file_manifest.chunk_hashes.first_mut() {
                *first_hash += "1";
            }
        }
        let file_meta = bundle.file_manifests.first().unwrap();
        let result = bundle.read_and_validate_file(file_meta);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_bundle() {
        let mut bundle = simple_bundle();
        let result = bundle.validate_local_bundle();
        assert!(result.is_ok());

        // Add tests for failure cases
        if let Some(file_meta) = bundle.file_manifests.first_mut() {
            if let Some(first_hash) = file_meta.file_manifest.chunk_hashes.first_mut() {
                *first_hash += "1";
            }
        }
        let result = bundle.validate_local_bundle();
        assert!(result.is_err());
    }
}
