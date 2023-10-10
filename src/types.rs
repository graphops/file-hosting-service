use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subfile {
    magnet_link: String,
    file_type: String,
    identifier: String,
    block_range: BlockRange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockRange {
    start_block: Option<u64>,
    end_block: Option<u64>,
}

