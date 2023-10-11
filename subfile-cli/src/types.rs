use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subfile {
    pub magnet_link: String,
    pub file_type: String,
    pub version: String,
    pub identifier: String,
    pub trackers: Vec<String>,
    pub block_range: BlockRange,
}

impl From<SeedCreationArg> for Subfile {
    fn from(arg: SeedCreationArg) -> Self {
        // create torrent
        // let torrent_file = arg.create_torrent();

        // generate magnet link
        let magnet_link = arg.generate_magnet_link();
        Self {
            magnet_link,
            file_type: arg.file_type,
            version: arg.version,
            identifier: arg.identifier,
            trackers: arg.trackers,
            block_range: arg.block_range,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockRange {
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeedCreationArg {
    pub file_path: String,
    pub file_type: String,
    pub version: String,
    pub identifier: String,
    pub trackers: Vec<String>,
    pub block_range: BlockRange,
}

impl SeedCreationArg {
    pub fn build(
        file_path: String,
        file_type: String,
        version: String,
        identifier: String,
        trackers: Vec<String>,
        start_block: Option<u64>,
        end_block: Option<u64>,
    ) -> Self {
        Self {
            file_path,
            file_type,
            version,
            identifier,
            trackers,
            block_range: BlockRange {
                start_block,
                end_block,
            },
        }
    }

    // pub fn generate_magnet_link(&self) -> String {
    //     // Placeholder: Replace with actual logic to generate magnet link
    //     format!("magnet:?xt=urn:btih:HASH&dn={}", self.file_path)
    // }
}

// #[derive(ValueEnum, Clone, Debug, Serialize, Deserialize, Default)]
// pub enum FileType {
//     #[default]
//     SqlSnapshot,
//     Flatfiles,
// }

// impl FromStr for FileType {
//     type Err = &'static str;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "sql_snapshot" => Ok(FileType::SqlSnapshot),
//             "flatfiles" => Ok(FileType::Flatfiles),
//             _ => Err("Invalid file type"),
//         }
//     }
// }
