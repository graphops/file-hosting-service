use rand::seq::IteratorRandom;
use rand::{distributions::Alphanumeric, Rng};
use std::env;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::config::init_tracing;
use crate::subfile::{BlockRange, ChunkFile, FileMetaInfo, Subfile, SubfileManifest};

pub const CHUNK_SIZE: u64 = 1024 * 1024; // Define the chunk size, e.g., 1 MB

// Helper function to create a temporary file with random content of a specified size
pub fn create_random_temp_file(size: usize) -> std::io::Result<(NamedTempFile, String)> {
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
pub fn create_temp_file(content: &[u8]) -> std::io::Result<(tempfile::NamedTempFile, String)> {
    let mut temp_file = tempfile::NamedTempFile::new()?;
    temp_file.write_all(content)?;
    let temp_path = temp_file.path().to_str().unwrap().to_string();
    Ok((temp_file, temp_path))
}

// Helper function to modify a single element at a random position
pub fn modify_random_element(matrix: &mut Vec<Vec<u8>>) -> Vec<Vec<u8>> {
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

pub fn simple_chunk_file() -> ChunkFile {
    ChunkFile {
        file_name: "0017234600.dbin.zst".to_string(),
        total_bytes: 26359000,
        chunk_size: 1048576,
        chunk_hashes: [
            "MIHJRsl2+fhrbTX9XQxR5/THummB4NfjSdlpUiwc4KM=".to_string(),
            "ETtu0ZbJOg/HCfpcWVafBLTo52tTplf7XaI7FC28f3g=".to_string(),
            "egmYeJKDDLUb4w5V2MWtWa+7OYU2qkKEjNtpi4beYFQ=".to_string(),
            "IXF0ko986rGGShWasNbvUfi42z9HzILfum+5saZORNE=".to_string(),
            "EWsdOfZ4Kxyt2gQz2atAqxVyW+uNQrVRDV5B/YDWNqU=".to_string(),
            "SDbCTKYd0yXZjFHYx5jBL1f4sXOoMbe+XHbBGB4lr/c=".to_string(),
            "bFmY9ZX9kNDX2IOiNFpQ3PXpANgWB7sT13VzQzFoCW4=".to_string(),
            "0gJv2+ugSEu5YObHCGMlPachg4a1rBLhjRC6h9vypUo=".to_string(),
            "Mb0sJCNBlDsX79lgS5upYMxE/Ogur3k5w8zV56xsujc=".to_string(),
            "mGi8bm5zqXyqNffmMKZxeMrOhCRD/a/qEhsyfyEoSWo=".to_string(),
            "54oAlwwO5AtDC66rsr54jQg4nzeBhILx0Yu3WRkposs=".to_string(),
            "mJNLGP5xg9crtEEodz86jEdH0/tI/8EN+O/EuIdMkso=".to_string(),
            "8yL8ga/aLbe+h/SNvdbX3RY1fN4U1P7nAHhoWTuuZKY=".to_string(),
            "Fg4OPNn5UH86DKpxKW0UsSj8qZ89Uw79zb+/xf1kajk=".to_string(),
            "rT3gyX+48dbOUuthys0xAniCDmAtal7Zur/ajztyVOk=".to_string(),
            "+gW9KCoZfEobqis9S6p4aNZoCeKLldUz/CzKn4k6hxs=".to_string(),
            "y3Z+xH33uexkeGZxVe1eI0oYzUrrkeEJcgdzYjcwt3w=".to_string(),
            "SAPD+TVgBeE9eAWe8kbzTGQYlDdfcmz9kmatByu4Bos=".to_string(),
            "+X30R3P65u5ealfn+zoypJEfodCHxxC98g3RMKgOmVg=".to_string(),
            "fFt5P452EBAPWeRCwLy+Na0BHRU0dLANmsceSroMQ2U=".to_string(),
            "skkMxFfH6yhrcaePxsOR6ux8LpFhcJ3V+dMXr9NeyR8=".to_string(),
            "p4G97Y6W/FQ8nR0lfz4YwKyPDQEsU8Ix5Rk6MM9h1x4=".to_string(),
            "OYMcj2nPkTghVJRFcgQt7MvurRC5a8tSfe4bst67LfI=".to_string(),
            "1nMl3Q0EdbpXxlZFsiKYOz/qr+MfcIosJMsEOJgCWJ4=".to_string(),
            "K2emdcJC2feqUjlK8C599XQqyAqbaY9dHwgzn+MmFp0=".to_string(),
            "LjPfv0JdXsPixi7LxdrcjVAVknRCUq9yDUVpGKOF3Sw=".to_string(),
        ]
        .to_vec(),
    }
}

pub fn simple_subfile() -> Subfile {
    Subfile {
        ipfs_hash: "QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ".to_string(),
        local_path: "../example-file/".into(),
        manifest: SubfileManifest {
            files: [FileMetaInfo {
                name: "0017234600.dbin.zst".to_string(),
                hash: "QmadNB1AQnap3czUime3gEETBNUj7HHzww6hVh5F6w7Boo".to_string(),
            }]
            .to_vec(),
            file_type: "flatfiles".to_string(),
            spec_version: "0.0.0".to_string(),
            description: "random flatfiles".to_string(),
            chain_id: "0".to_string(),
            block_range: BlockRange {
                start_block: None,
                end_block: None,
            },
        },
        chunk_files: [simple_chunk_file()].to_vec(),
    }
}

// pub const SUBFILE_MANIFEST = r#"files:
// - name: example-create-17686085.dbin
//   hash: QmSgzLLsQzdRAQRA2d7X3wqLEUTBLSbRe2tqv9rJBy7Wqv
// - name: 0017234500.dbin.zst
//   hash: Qmexz4ZariJteKHHXMxsSeSjvyLZf7SUWz77bsvLUQG1Vn
// - name: 0017234600.dbin.zst
//   hash: QmadNB1AQnap3czUime3gEETBNUj7HHzww6hVh5F6w7Boo
// - name: 0017686111-c1ed20dc4cffd7bd-ebfe6d2b6a25625a-17686021-default.dbin
//   hash: QmSEDiCKax7rjxS3kvGJ3dPdHkm2bztFZkR5KDqfpgyuQw
// - name: 0017686115-f8d105f60fa2e78d-7d23a3e458beaff1-17686021-default.dbin
//   hash: QmVx3JX5TNrSqMKyP5xQJ2CYmcqG4VaBdPnbji3PuvUFx6
// file_type: flatfiles
// spec_version: 0.0.0
// description: random flatfiles
// chain_id: '0'
// block_range:
//   start_block: null
//   end_block: null";

pub fn init_logger() {
    env::set_var("RUST_LOG", "warn,subfile_exchange=trace");
    init_tracing(String::from("pretty")).unwrap();
}
