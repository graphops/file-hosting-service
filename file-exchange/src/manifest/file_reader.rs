use bytes::Bytes;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::errors::Error;

// Read a chunk from the file at the file_path from specified start and end bytes
pub fn read_chunk(file_path: &Path, (start, end): (u64, u64)) -> Result<Bytes, Error> {
    let mut file = File::open(file_path).map_err(Error::FileIOError)?;

    let file_size = file
        .metadata()
        .map(|d| d.len())
        .map_err(Error::FileIOError)?;

    tracing::trace!(start, end, file_size, "Range validity check");
    if start >= file_size || end >= file_size {
        return Err(Error::InvalidRange(format!(
            "Range ({:#?}, {:#?}) out of bound for file size {:#?}",
            start, end, file_size
        )));
    }

    let length = end - start + 1;

    match file.seek(SeekFrom::Start(start)) {
        Ok(_) => {
            tracing::trace!("File seek to start at {:#?}", start)
        }
        Err(e) => return Err(Error::FileIOError(e)),
    }

    let mut buffer = vec![0; length as usize];
    match file.read_exact(&mut buffer) {
        Ok(_) => {}
        Err(e) => return Err(Error::FileIOError(e)),
    };

    Ok(buffer.into())
}

pub fn format_path(read_dir: &str, file_name: &str) -> String {
    format!(
        "{}{}{}",
        read_dir,
        if read_dir.ends_with('/') { "" } else { "/" },
        file_name
    )
}
