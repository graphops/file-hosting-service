use anyhow::anyhow;
use bytes::Bytes;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::file_hasher::{verify_chunk, ChunkFile};
use crate::ipfs::IpfsClient;
use crate::subfile_reader::read_subfile;
use crate::types::Subfile;

// TODO: REFACTOR; read chunk can be further refactors, check for valid path, and use in serve_file_range
// Read a chunk from the file at the file_path from specified start and end bytes
pub fn read_chunk(file_path: &Path, (start, end): (u64, u64)) -> Result<Bytes, anyhow::Error> {
    let mut file =
        File::open(file_path).map_err(|e| anyhow!("Cannot access file: {:#?}", e.to_string()))?;

    let file_size = file
        .metadata()
        .map(|d| d.len())
        .map_err(|e| anyhow!("Cannot get file metadata: {:#?}", e.to_string()))?;

    tracing::debug!(start, end, file_size, "Range validity check");
    if start >= file_size || end >= file_size {
        return Err(anyhow!(
            "Range ({:#?}, {:#?}) out of bound for file size {:#?}",
            start,
            end,
            file_size
        ));
    }

    let length = end - start + 1;

    match file.seek(SeekFrom::Start(start)) {
        Ok(_) => {
            tracing::trace!("File seek to start at {:#?}", start)
        }
        Err(e) => return Err(anyhow!("Failed to seek file start: {:#?}", e.to_string())),
    }

    let mut buffer = vec![0; length as usize];
    match file.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(e) => return Err(anyhow!("Failed to react exact bytes: {:#?}", e.to_string())),
    };

    Ok(buffer.into())
}

/// Validate the local files against a given subfile specification
pub async fn validate_local_subfile(
    client: &IpfsClient,
    ipfs_hash: String,
    local_path: PathBuf,
) -> Result<Subfile, anyhow::Error> {
    let subfile = read_subfile(client, &ipfs_hash, local_path).await?;
    tracing::debug!(
        subfile = tracing::field::debug(&subfile),
        "Read and verify subfile"
    );

    // Read all files in subfile to verify locally. This may cause a long initialization time
    for chunk_file in &subfile.chunk_files {
        if let Err(e) = read_and_validate_file(&subfile, chunk_file) {
            panic!("Damn, {}. Fix before continuing", e);
        };
    }

    tracing::debug!("Successfully verified the local serving files");
    Ok(subfile)
}

/// Read and validate file
pub fn read_and_validate_file(
    subfile: &Subfile,
    chunk_file: &ChunkFile,
) -> Result<(), anyhow::Error> {
    // read file by chunk_file.file_name
    let mut file_path = subfile.local_path.clone();
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
