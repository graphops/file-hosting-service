use anyhow::anyhow;
use bytes::Bytes;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

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
