use merkle_cbt::merkle_tree::{Merge, CBMT};
use merkle_cbt::{MerkleProof, MerkleTree};
use sha2::{Digest, Sha256};

use serde_yaml::to_string;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub const CHUNK_SIZE: usize = 1024 * 1024; // Define the chunk size, e.g., 1 MB

#[derive(Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct ChunkFile {
    // pub merkle_root: String,
    pub file_name: String,
    pub total_bytes: u64,
    pub chunk_size: u64,
    pub chunk_hashes: Vec<String>,
}

fn hash_chunk(chunk: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(chunk);
    let hash = hasher.finalize().to_vec();
    tracing::debug!(hash = tracing::field::debug(&hash), "Chunk hash");
    let hash_str = base64::encode(hash);
    tracing::debug!(hash_str = tracing::field::debug(&hash_str), "Chunk hash");
    hash_str
}

/// Read the file at file_path and chunk the file into bytes
fn chunk_file(file_path: &Path) -> Result<(u64, Vec<Vec<u8>>), anyhow::Error> {
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

    tracing::debug!(file = tracing::field::debug(file_path), total_bytes, num_chunks = chunks.len(), "Chunked file");
    Ok((total_bytes.try_into().unwrap(), chunks))
}

pub struct MergeU8;

impl Merge for MergeU8 {
    type Item = Vec<u8>;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into_iter().collect()
    }
}

pub type MerkleProofU8 = MerkleProof<Vec<u8>, MergeU8>;
pub type MerkleTreeU8 = MerkleTree<Vec<u8>, MergeU8>;
pub type CBMTU8 = CBMT<Vec<u8>, MergeU8>;

pub fn merkle_root(leaves: &[Vec<u8>]) -> Vec<u8> {
    CBMTU8::build_merkle_root(leaves)
}

pub fn build_merkle_tree(leaves: Vec<Vec<u8>>) -> MerkleTreeU8 {
    CBMTU8::build_merkle_tree(&leaves)
}

pub fn build_merkle_proof(leaves: &[Vec<u8>], indices: &[u32]) -> Option<MerkleProofU8> {
    CBMTU8::build_merkle_proof(leaves, indices)
}

/// Let chunk_file be
/// - file name
/// - chunk size in bytes
/// - total bytes
/// - list of hashes in order of the file
// pub fn create_chunk_file(merkle_tree: &MerkleTreeU8) -> ChunkFile {
pub fn create_chunk_file(read_dir: &str, file_name: &str) -> Result<ChunkFile, anyhow::Error> {
    let file_path = format!("{}/{}", read_dir, file_name);
    // let merkle_root = hex::encode(merkle_tree.root());
    // let chunk_hashes: Vec<String> = merkle_tree.nodes().iter().map(hex::encode).collect();
    let (total_bytes, chunks) = chunk_file(Path::new(&file_path))?;

    let chunk_hashes: Vec<String> = chunks.iter().map(|c| hash_chunk(&c)).collect();

    Ok(ChunkFile {
        file_name: file_name.to_string(),
        total_bytes,
        chunk_size: CHUNK_SIZE as u64,
        chunk_hashes,
    })
}

pub fn write_chunk_file(read_dir: &str, file_name: &str) -> Result<String, anyhow::Error> {
    // let (_, chunks) = chunk_file(Path::new(&file_path))?;
    // let merkle_tree = build_merkle_tree(chunks);
    // let chunk_file = create_chunk_file(&merkle_tree);
    let chunk_file = create_chunk_file(read_dir, file_name)?;

    tracing::trace!(file = tracing::field::debug(&chunk_file), "Created chunk file");

    let yaml = to_string(&chunk_file)?;
    // TODO: consider storing a local copy
    // let mut output_file = File::create(file_path)?;
    // output_file.write_all(yaml.as_bytes())?;

    Ok(yaml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::*;

    #[test]
    fn test_same_files_produce_same_hash() {
        let content = b"Hello, world!";
        let (temp_file1, temp_path1) = create_temp_file(content).unwrap();
        let (temp_file2, temp_path2) = create_temp_file(content).unwrap();

        // let merkle_tree1 = build_merkle_tree(chunks1);
        // let merkle_tree2 = build_merkle_tree(chunks2);

        // assert_eq!(merkle_tree1.root(), merkle_tree2.root());
        let path1 = Path::new(&temp_path1);
        let path2 = Path::new(&temp_path2);
        let readdir1 = path1.parent().unwrap().to_str().unwrap();
        let readdir2 = path2.parent().unwrap().to_str().unwrap();
        let file_name1 = path1.file_name().unwrap().to_str().unwrap();
        let file_name2 = path2.file_name().unwrap().to_str().unwrap();

        // produce the same chunk file
        let chunk_file1 = create_chunk_file(readdir1, file_name1).unwrap();
        let chunk_file2 = create_chunk_file(readdir2, file_name2).unwrap();

        assert_eq!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);
        drop(temp_file2);
    }

    #[test]
    fn test_different_files_produce_different_hash() {
        let content1 = b"Hello, world!";
        let content2 = b"Goodbye, world!";
        let (temp_file1, temp_path1) = create_temp_file(content1).unwrap();
        let (temp_file2, temp_path2) = create_temp_file(content2).unwrap();

        let path1 = Path::new(&temp_path1);
        let path2 = Path::new(&temp_path2);
        let readdir1 = path1.parent().unwrap().to_str().unwrap();
        let readdir2 = path2.parent().unwrap().to_str().unwrap();
        let file_name1 = path1.file_name().unwrap().to_str().unwrap();
        let file_name2 = path2.file_name().unwrap().to_str().unwrap();

        // produce different chunk file
        let chunk_file1 = create_chunk_file(readdir1, file_name1).unwrap();
        let chunk_file2 = create_chunk_file(readdir2, file_name2).unwrap();

        assert_ne!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);
        drop(temp_file2);
    }

    #[test]
    fn test_big_size_same_file() {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size).unwrap();

        let path = Path::new(&temp_path1);
        let readdir = path.parent().unwrap().to_str().unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        // produce the same chunk file
        let chunk_file1 = create_chunk_file(readdir, file_name).unwrap();
        let chunk_file2 = create_chunk_file(readdir, file_name).unwrap();

        assert_eq!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);
    }

    #[test]
    fn test_big_size_different_file() {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size).unwrap();

        let path1 = Path::new(&temp_path1);
        let readdir1 = path1.parent().unwrap().to_str().unwrap();
        let file_name1 = path1.file_name().unwrap().to_str().unwrap();

        let (_, chunks1) = chunk_file(Path::new(&temp_path1)).unwrap();
        // Modify a byte at an arbitrary postiion
        let chunks2 = modify_random_element(&mut chunks1.clone());
        assert_ne!(chunks2, chunks1);

        let (_, temp_path2) = create_temp_file(&chunks2.concat()).unwrap();
        let path2 = Path::new(&temp_path2);
        let readdir2 = path2.parent().unwrap().to_str().unwrap();
        let file_name2 = path2.file_name().unwrap().to_str().unwrap();

        // produce different chunk file
        let chunk_file1 = create_chunk_file(readdir1, file_name1).unwrap();
        let chunk_file2 = create_chunk_file(readdir2, file_name2).unwrap();

        assert_ne!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);
    }
}
