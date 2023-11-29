use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    file_hasher::{hash_chunk, verify_chunk},
    file_reader::{chunk_file, format_path, read_chunk},
    types::CHUNK_SIZE,
};

/* Public Manifests */

/// Better mapping of files and chunk files
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubfileManifest {
    pub files: Vec<FileMetaInfo>,
    pub file_type: String,
    pub spec_version: String,
    pub description: String,
    pub chain_id: String,
    pub block_range: BlockRange,
    // pub identifier: String,
    // pub publisher_url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileMetaInfo {
    pub name: String,
    pub hash: String,
    // pub file_link: String,
    // pub file_name: String,
    // pub block_range: BlockRange,
}

/* Chunk file */
#[derive(Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq, Clone)]
pub struct ChunkFile {
    // pub merkle_root: String,
    pub file_name: String,
    pub total_bytes: u64,
    pub chunk_size: u64,
    pub chunk_hashes: Vec<String>,
}

impl ChunkFile {
    // pub fn create_chunk_file(merkle_tree: &MerkleTreeU8) -> ChunkFile {
    pub fn new(read_dir: &str, file_name: &str) -> Result<ChunkFile, anyhow::Error> {
        let file_path = format_path(read_dir, file_name);
        // let merkle_root = hex::encode(merkle_tree.root());
        // let chunk_hashes: Vec<String> = merkle_tree.nodes().iter().map(hex::encode).collect();
        println!("Reading file: {:#?}", &file_path);
        let (total_bytes, chunks) = chunk_file(Path::new(&file_path))?;
        println!("toatla bytes: {:#?}", total_bytes);

        let chunk_hashes: Vec<String> = chunks.iter().map(|c| hash_chunk(c)).collect();

        Ok(ChunkFile {
            file_name: file_name.to_string(),
            total_bytes,
            chunk_size: CHUNK_SIZE as u64,
            chunk_hashes,
        })
    }
}

/* Subfile - packaging of chunk files mapped into local files */
//TODO: Add GraphQL derivation
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

impl Subfile {
    /// Validate the local files against a given subfile specification
    pub fn validate_local_subfile(&self) -> Result<&Self, anyhow::Error> {
        tracing::debug!(
            subfile = tracing::field::debug(self),
            "Read and verify subfile"
        );

        // Read all files in subfile to verify locally. This may cause a long initialization time
        for chunk_file in &self.chunk_files {
            if let Err(e) = self.read_and_validate_file(chunk_file) {
                panic!("Damn, {}. Fix before continuing", e);
            };
        }

        tracing::debug!("Successfully verified the local serving files");
        Ok(self)
    }

    /// Read and validate file
    pub fn read_and_validate_file(&self, chunk_file: &ChunkFile) -> Result<(), anyhow::Error> {
        // read file by chunk_file.file_name
        let mut file_path = self.local_path.clone();
        file_path.push(chunk_file.file_name.clone());
        tracing::trace!(
            file_path = tracing::field::debug(&file_path),
            chunk_file = tracing::field::debug(&chunk_file),
            "Verify file"
        );

        // loop through chunk file  byte range
        for i in 0..(chunk_file.total_bytes / chunk_file.chunk_size + 1) {
            // read range
            let start = i * chunk_file.chunk_size;
            let end = u64::min(start + chunk_file.chunk_size, chunk_file.total_bytes) - 1;
            tracing::trace!(
                i,
                start_byte = tracing::field::debug(&start),
                end_byte = tracing::field::debug(&end),
                "Verify chunk index"
            );
            let chunk_hash = chunk_file.chunk_hashes[i as usize].clone();

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
                return Err(anyhow::anyhow!(
                    "Failed to validate the local version of file {}",
                    chunk_file.file_name
                ));
            }
        }
        Ok(())
    }
}
