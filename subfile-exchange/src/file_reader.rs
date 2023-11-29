use anyhow::anyhow;
use bytes::Bytes;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::types::CHUNK_SIZE;

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

/// Read the file at file_path and chunk the file into bytes
pub fn chunk_file(file_path: &Path) -> Result<(u64, Vec<Vec<u8>>), anyhow::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut chunks = Vec::new();
    let mut total_bytes = 0;

    loop {
        let mut buffer = vec![0; CHUNK_SIZE];
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        total_bytes += bytes_read;
        buffer.truncate(bytes_read);
        chunks.push(buffer);
    }

    tracing::debug!(
        file = tracing::field::debug(file_path),
        total_bytes,
        num_chunks = chunks.len(),
        "Chunked file"
    );
    Ok((total_bytes.try_into().unwrap(), chunks))
}

pub fn format_path(read_dir: &str, file_name: &str) -> String {
    format!(
        "{}{}{}",
        read_dir,
        if read_dir.ends_with('/') { "" } else { "/" },
        file_name
    )
}
