use merkle_cbt::merkle_tree::{Merge, CBMT};
use merkle_cbt::{MerkleProof, MerkleTree};
use sha2::{Digest, Sha256};

use serde_yaml::to_string;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

const CHUNK_SIZE: usize = 1024 * 1024; // Define the chunk size, e.g., 1 MB

#[derive(Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct ChunkFile {
    pub merkle_root: String,
    pub chunks: Vec<String>,
}

fn hash_chunk(chunk: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(chunk);
    hasher.finalize().to_vec()
}

/// Read the file at file_path and chunk the file into bytes
fn chunk_file(file_path: &Path) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut chunks = Vec::new();

    loop {
        let mut buffer = vec![0; CHUNK_SIZE];
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        buffer.truncate(bytes_read);
        chunks.push(buffer);
    }

    Ok(chunks)
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

pub fn create_chunk_file(merkle_tree: &MerkleTreeU8) -> ChunkFile {
    let merkle_root = hex::encode(merkle_tree.root());
    let chunk_hashes: Vec<String> = merkle_tree.nodes().iter().map(hex::encode).collect();

    ChunkFile {
        merkle_root,
        chunks: chunk_hashes,
    }
}

pub fn write_chunk_file(file_path: &str) -> Result<String, anyhow::Error> {
    let chunks = chunk_file(Path::new(&file_path))?;

    let merkle_tree = build_merkle_tree(chunks);

    let chunk_file = create_chunk_file(&merkle_tree);
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

        let chunks1 = chunk_file(Path::new(&temp_path1)).unwrap();
        let chunks2 = chunk_file(Path::new(&temp_path2)).unwrap();

        let merkle_tree1 = build_merkle_tree(chunks1);
        let merkle_tree2 = build_merkle_tree(chunks2);

        assert_eq!(merkle_tree1.root(), merkle_tree2.root());

        // produce the same chunk file
        let chunk_file1 = create_chunk_file(&merkle_tree1);
        let chunk_file2 = create_chunk_file(&merkle_tree2);

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

        let chunks1 = chunk_file(Path::new(&temp_path1)).unwrap();
        let chunks2 = chunk_file(Path::new(&temp_path2)).unwrap();

        let merkle_tree1 = build_merkle_tree(chunks1);
        let merkle_tree2 = build_merkle_tree(chunks2);

        assert_ne!(merkle_tree1.root(), merkle_tree2.root());

        // produce the same chunk file
        let chunk_file1 = create_chunk_file(&merkle_tree1);
        let chunk_file2 = create_chunk_file(&merkle_tree2);

        assert_ne!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);
        drop(temp_file2);
    }

    #[test]
    fn test_big_size_same_file() {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size).unwrap();

        let chunks1 = chunk_file(Path::new(&temp_path1)).unwrap();
        let chunks2 = chunk_file(Path::new(&temp_path1)).unwrap();

        let merkle_tree1 = build_merkle_tree(chunks1);
        let merkle_tree2 = build_merkle_tree(chunks2);

        assert_eq!(merkle_tree1.root(), merkle_tree2.root());

        // produce the same chunk file
        let chunk_file1 = create_chunk_file(&merkle_tree1);
        let chunk_file2 = create_chunk_file(&merkle_tree2);

        assert_eq!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);
    }

    #[test]
    fn test_big_size_different_file() {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size).unwrap();

        let chunks1 = chunk_file(Path::new(&temp_path1)).unwrap();
        // Modify a byte at an arbitrary postiion
        let chunks2 = modify_random_element(&mut chunks1.clone());
        assert_ne!(chunks2, chunks1);

        let merkle_tree1 = build_merkle_tree(chunks1);
        let merkle_tree2 = build_merkle_tree(chunks2);

        assert_ne!(merkle_tree1.root(), merkle_tree2.root());

        // produce the same chunk file
        let chunk_file1 = create_chunk_file(&merkle_tree1);
        let chunk_file2 = create_chunk_file(&merkle_tree2);

        assert_ne!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);
    }
}
