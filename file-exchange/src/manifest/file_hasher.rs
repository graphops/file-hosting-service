use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use merkle_cbt::merkle_tree::{Merge, CBMT};
use merkle_cbt::{MerkleProof, MerkleTree};
use sha2::{Digest, Sha256};

pub fn hash_chunk(chunk: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(chunk);
    let hash = hasher.finalize().to_vec();
    let hash_str = general_purpose::STANDARD.encode(hash);
    tracing::trace!(hash_str = tracing::field::debug(&hash_str), "Chunk hash");
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

/// Verify a vector of Bytes against a canonical hash
pub fn verify_chunk(data: &Bytes, chunk_hash: &str) -> bool {
    let downloaded_chunk_hash = hash_chunk(data);
    downloaded_chunk_hash == chunk_hash
}

#[cfg(test)]
mod tests {
    use crate::{manifest::file_reader::chunk_file, manifest::FileManifest, test_util::*};
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

        // produce the same file manifest
        let file_manifest1 = FileManifest::new(readdir1, file_name1, CHUNK_SIZE).unwrap();
        let file_manifest2 = FileManifest::new(readdir2, file_name2, CHUNK_SIZE).unwrap();

        assert_eq!(file_manifest1.chunk_hashes, file_manifest2.chunk_hashes);

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

        // produce different file manifest
        let file_manifest1 = FileManifest::new(readdir1, file_name1, CHUNK_SIZE).unwrap();
        let file_manifest2 = FileManifest::new(readdir2, file_name2, CHUNK_SIZE).unwrap();

        assert_ne!(file_manifest1.chunk_hashes, file_manifest2.chunk_hashes);

        // Clean up
        drop(temp_file1);
        drop(temp_file2);
    }

    #[test]
    fn test_big_size_same_file() {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size as usize).unwrap();

        let path = Path::new(&temp_path1);
        let readdir = path.parent().unwrap().to_str().unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        // produce the same file manifest
        let file_manifest1 = FileManifest::new(readdir, file_name, file_size).unwrap();
        let file_manifest2 = FileManifest::new(readdir, file_name, file_size).unwrap();

        assert_eq!(file_manifest1, file_manifest2);

        // Clean up
        drop(temp_file1);
    }

    #[test]
    fn test_big_size_different_file() {
        let file_size = CHUNK_SIZE * 25;
        let (temp_file1, temp_path1) = create_random_temp_file(file_size as usize).unwrap();

        let path1 = Path::new(&temp_path1);
        let readdir1 = path1.parent().unwrap().to_str().unwrap();
        let file_name1 = path1.file_name().unwrap().to_str().unwrap();

        let (_, chunks1) = chunk_file(Path::new(&temp_path1), file_size).unwrap();
        // Modify a byte at an arbitrary postiion
        let chunks2 = modify_random_element(&mut chunks1.clone());
        assert_ne!(chunks2, chunks1);

        let (temp_file2, temp_path2) = create_temp_file(&chunks2.concat()).unwrap();
        let path2 = Path::new(&temp_path2);
        let readdir2 = path2.parent().unwrap().to_str().unwrap();
        let file_name2 = path2.file_name().unwrap().to_str().unwrap();

        // produce different file manifest
        let file_manifest1 = FileManifest::new(readdir1, file_name1, file_size).unwrap();
        let file_manifest2 = FileManifest::new(readdir2, file_name2, file_size).unwrap();

        assert_ne!(file_manifest1.chunk_hashes, file_manifest2.chunk_hashes);

        // Clean up
        drop(temp_file1);
        drop(temp_file2);
    }
}
