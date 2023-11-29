use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    file_hasher::{verify_chunk, ChunkFile},
    file_reader::read_chunk,
    publisher::SubfileManifest,
};

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

#[derive(Serialize, Deserialize)]
pub struct Health {
    pub healthy: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Operator {
    #[serde(alias = "publicKey")]
    pub public_key: String,
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
