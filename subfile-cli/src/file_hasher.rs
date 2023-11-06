extern crate merkle_cbt;
extern crate serde_yaml;
extern crate sha2;

use merkle_cbt::merkle_tree::{Merge, CBMT};
use merkle_cbt::{MerkleProof, MerkleTree};
use sha2::{Digest, Sha256};

use serde_yaml::to_string;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Write};
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

fn chunk_file(file_path: &Path) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
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
        hasher.finalize().into_iter().collect() //.to_vec()
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

pub fn write_chunk_file(file_path: &Path, merkle_tree: &MerkleTreeU8) -> Result<(), Box<dyn Error>> {
    let chunk_file = create_chunk_file(merkle_tree);
    let yaml = to_string(&chunk_file)?;
    let mut output_file = File::create(file_path)?;
    output_file.write_all(yaml.as_bytes())?;

    Ok(())
}

/// Takes file_path, create chunk_file, build merkle tree, publish, write to output
pub fn publish_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let file_path = Path::new(file_path);
    let chunks = chunk_file(file_path)?;
    let merkle_tree = build_merkle_tree(chunks);
    let output = Path::new("chunk_file.yaml");
    //TODO: publish the chunk file to IPFS or another distributed file system here
    write_chunk_file(output, &merkle_tree)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::seq::IteratorRandom;
    use rand::{distributions::Alphanumeric, Rng};
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Helper function to create a temporary file with random content of a specified size
    fn create_random_temp_file(size: usize) -> std::io::Result<(NamedTempFile, String)> {
        let mut temp_file = NamedTempFile::new()?;
        let content: Vec<u8> = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size)
            .collect();
        temp_file.write_all(&content)?;
        let temp_path = temp_file.path().to_str().unwrap().to_string();
        Ok((temp_file, temp_path))
    }

    // Helper function to create a temporary file with given content
    fn create_temp_file(content: &[u8]) -> std::io::Result<(tempfile::NamedTempFile, String)> {
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(content)?;
        let temp_path = temp_file.path().to_str().unwrap().to_string();
        Ok((temp_file, temp_path))
    }

    // Helper function to modify a single element at a random position
    fn modify_random_element(matrix: &mut Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let mut rng = rand::thread_rng();

        if let Some(outer_idx) = matrix.iter().enumerate().choose(&mut rng) {
            let (outer_idx, inner_vec) = outer_idx;
            if let Some(inner_idx) = inner_vec.iter().enumerate().choose(&mut rng) {
                let (inner_idx, _) = inner_idx;
                matrix[outer_idx][inner_idx] = matrix[outer_idx][inner_idx].wrapping_add(1);
            }
        }

        matrix.to_vec()
    }

    #[test]
    fn test_same_files_produce_same_hash() -> Result<(), Box<dyn Error>> {
        let content = b"Hello, world!";
        let (temp_file1, temp_path1) = create_temp_file(content)?;
        let (temp_file2, temp_path2) = create_temp_file(content)?;

        let chunks1 = chunk_file(Path::new(&temp_path1))?;
        let chunks2 = chunk_file(Path::new(&temp_path2))?;

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

        Ok(())
    }

    #[test]
    fn test_different_files_produce_different_hash() -> Result<(), Box<dyn Error>> {
        let content1 = b"Hello, world!";
        let content2 = b"Goodbye, world!";
        let (temp_file1, temp_path1) = create_temp_file(content1)?;
        let (temp_file2, temp_path2) = create_temp_file(content2)?;

        let chunks1 = chunk_file(Path::new(&temp_path1))?;
        let chunks2 = chunk_file(Path::new(&temp_path2))?;

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

        Ok(())
    }

    #[test]
    fn test_big_size_same_file() -> Result<(), Box<dyn Error>> {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size)?;

        let chunks1 = chunk_file(Path::new(&temp_path1))?;
        let chunks2 = chunk_file(Path::new(&temp_path1))?;

        let merkle_tree1 = build_merkle_tree(chunks1);
        let merkle_tree2 = build_merkle_tree(chunks2);

        assert_eq!(merkle_tree1.root(), merkle_tree2.root());

        // produce the same chunk file
        let chunk_file1 = create_chunk_file(&merkle_tree1);
        let chunk_file2 = create_chunk_file(&merkle_tree2);

        assert_eq!(chunk_file1, chunk_file2);

        // Clean up
        drop(temp_file1);

        Ok(())
    }

    #[test]
    fn test_big_size_different_file() -> Result<(), Box<dyn Error>> {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size)?;

        let chunks1 = chunk_file(Path::new(&temp_path1))?;
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

        Ok(())
    }
}
