use rand::seq::IteratorRandom;
use rand::{distributions::Alphanumeric, Rng};
use std::io::Write;
use tempfile::NamedTempFile;

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
