use bytes::Bytes;
use merkle_cbt::merkle_tree::{Merge, CBMT};
use merkle_cbt::{MerkleProof, MerkleTree};
use sha2::{Digest, Sha256};

use crate::subfile::ChunkFile;
use serde_yaml::to_string;

pub fn hash_chunk(chunk: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(chunk);
    let hash = hasher.finalize().to_vec();
    tracing::debug!(hash = tracing::field::debug(&hash), "Chunk hash");
    //TODO: update to a better encoder
    let hash_str = base64::encode(hash);
    tracing::debug!(hash_str = tracing::field::debug(&hash_str), "Chunk hash");
    hash_str
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

pub fn write_chunk_file(read_dir: &str, file_name: &str) -> Result<String, anyhow::Error> {
    // let (_, chunks) = chunk_file(Path::new(&file_path))?;
    // let merkle_tree = build_merkle_tree(chunks);
    // let chunk_file = create_chunk_file(&merkle_tree);

    tracing::trace!(read_dir, file_name, "write_chunk_file",);

    let chunk_file = ChunkFile::new(read_dir, file_name)?;

    tracing::trace!(
        file = tracing::field::debug(&chunk_file),
        "Created chunk file"
    );

    let yaml = to_string(&chunk_file)?;
    // TODO: consider storing a local copy
    // let mut output_file = File::create(file_path)?;
    // output_file.write_all(yaml.as_bytes())?;

    Ok(yaml)
}

/// Verify a vector of Bytes against a canonical hash
pub fn verify_chunk(data: &Bytes, chunk_hash: &str) -> bool {
    let downloaded_chunk_hash = hash_chunk(data);
    &downloaded_chunk_hash == chunk_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{file_reader::chunk_file, test_util::*, types::CHUNK_SIZE};
    use std::path::Path;

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
        let chunk_file1 = ChunkFile::new(readdir1, file_name1).unwrap();
        let chunk_file2 = ChunkFile::new(readdir2, file_name2).unwrap();

        assert_eq!(chunk_file1.chunk_hashes, chunk_file2.chunk_hashes);

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
        let chunk_file1 = ChunkFile::new(readdir1, file_name1).unwrap();
        let chunk_file2 = ChunkFile::new(readdir2, file_name2).unwrap();

        assert_ne!(chunk_file1.chunk_hashes, chunk_file2.chunk_hashes);

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
        let chunk_file1 = ChunkFile::new(readdir, file_name).unwrap();
        let chunk_file2 = ChunkFile::new(readdir, file_name).unwrap();

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

        let (temp_file2, temp_path2) = create_temp_file(&chunks2.concat()).unwrap();
        let path2 = Path::new(&temp_path2);
        let readdir2 = path2.parent().unwrap().to_str().unwrap();
        let file_name2 = path2.file_name().unwrap().to_str().unwrap();

        // produce different chunk file
        let chunk_file1 = ChunkFile::new(readdir1, file_name1).unwrap();
        let chunk_file2 = ChunkFile::new(readdir2, file_name2).unwrap();

        assert_ne!(chunk_file1.chunk_hashes, chunk_file2.chunk_hashes);

        // Clean up
        drop(temp_file1);
        drop(temp_file2);
    }
}
