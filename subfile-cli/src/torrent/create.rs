use std::{fs, io::Write, path::PathBuf};

use anyhow::anyhow;
use reqwest::{ResponseBuilderExt, Url};

use torrent_helper::{
    common::Lexiclean,
    consts,
    env::Env,
    hasher::Hasher,
    info::Info,
    input_target::InputTarget,
    into_usize::IntoUsize,
    magnet_link::MagnetLink,
    metainfo::Metainfo,
    options::Options,
    output_target::OutputTarget,
    piece_length_picker::PieceLengthPicker,
    subcommand::torrent::create::{create_content::CreateContent, Create},
    torrent_summary::TorrentSummary,
    use_color::UseColor,
    walker::Walker,
};

use tracing::{info, trace};

use crate::types::SeedCreationArg;

impl SeedCreationArg {
    //TODO: Add more confinguration for how torrent file is created
    pub fn generate_torrent_and_magnet_link(&self) -> Result<MagnetLink, anyhow::Error> {
        let path_buf: PathBuf = if let Some(path) = self.file_path.clone() {
            PathBuf::from(&path)
        } else {
            return Err(anyhow!("No magnet_link nor file path"));
        };

        let files = Walker::new(&path_buf).files()?;

        let piece_length = PieceLengthPicker::from_content_size(files.total_size());

        let hasher = Hasher::new(
            false, //md5 checksum cryptographic scheme is broken
            piece_length.as_piece_length()?.into_usize(),
            None, // no progress bar
        );

        let (mode, pieces) = hasher.hash_files(&files)?;
        let name = String::from(&self.file_name);

        let info = Info {
            name: name.clone(),
            piece_length,
            source: None,
            update_url: None,
            mode,
            pieces,
            private: None,
        };

        let metainfo = Metainfo {
            comment: None,
            encoding: Some(consts::ENCODING_UTF8.to_owned()),
            announce: self.trackers.first().cloned(),
            announce_list: None,
            nodes: None,
            creation_date: None,
            created_by: None,
            info,
        };

        let bytes = metainfo.serialize()?;

        //TODO: Add a dry run config flag
        let output = path_buf
            .join("..")
            .lexiclean()
            .join(format!("{}.torrent", name.clone()));
        // let output = OutputTarget::Path(CreateContent::torrent_path(&path_buf, &name));
        let mut open_options = fs::OpenOptions::new();

        //TODO: Add a force config flag
        open_options.write(true).create(true).truncate(true);

        open_options
            .open(output)
            .and_then(|mut file| file.write_all(&bytes))?;

        //TODO: Verify metainfo of the torrent file created
        // #[cfg(test)]
        // {
        //     let deserialized = bendy::serde::de::from_bytes::<Metainfo>(&bytes).unwrap();

        //     assert_eq!(deserialized, metainfo);

        //     let status = metainfo.verify(&path_buf, None)?;

        //     status.print(env)?;

        //     if !status.good() {
        //         return Err(Error::Verify);
        //     }
        // }

        info!("Generated Torrent file");

        let res = TorrentSummary::from_metainfo_lossy(metainfo.clone())?;
        trace!(
            summary = tracing::field::debug(&res),
            "Torrent summary"
        );

        let link = MagnetLink::from_metainfo_lossy(&metainfo)?;
        // let mut link = MagnetLink::from_metainfo_lossy(&metainfo)?;
        // for peer in self.peers {
        //     link.add_peer(peer);
        // }
        info!(link = tracing::field::debug(&link), "Magnet Link");

        //   if let OutputTarget::Path(path) = output {
        //     if self.open {
        //       Platform::open_file(&path)?;
        //     }
        //   }

        Ok(link)
    }
    // pub fn create_torrent_and_magnet_link(&self) -> Result<MagnetLink, anyhow::Error> {
    //     // input indicated by file_path
    //     let create_opt = Create::from(self.clone());

    //     let mut env = match Env::main() {
    //         Ok(env) => env,
    //         Err(err) => {
    //             eprintln!("{}", err);
    //             panic!("Cannot read env");
    //         }
    //     };
    //     let options = Options {
    //         quiet: false,
    //         unstable: false,
    //         use_color: UseColor::Auto,
    //         terminal: false,
    //     };

    //     // let magnet_link =
    //     Ok(create_opt.generate_magnet_link(&mut env, &options)?)

    //     // generate magnet link
    //     // let magnet_link = arg.generate_magnet_link();
    //     // Self {
    //     //     magnet_link,
    //     //     file_type: arg.file_type,
    //     //     version: arg.version,
    //     //     identifier: arg.identifier,
    //     //     trackers: arg.trackers,
    //     //     block_range: arg.block_range,
    //     // }
    // }

    // pub fn generate_magnet_link(&self) -> String {
    //     // Placeholder: Replace with actual logic to generate magnet link
    //     format!("magnet:?xt=urn:btih:HASH&dn={}", self.file_path)
    // }
}

impl From<SeedCreationArg> for Create {
    fn from(arg: SeedCreationArg) -> Self {
        // create torrent
        let trackers = arg
            .trackers
            .iter()
            .map(|t| Url::parse(t).ok())
            .collect::<Vec<Option<Url>>>();
        let primary_tracker = if let Some(t) = arg.trackers.first() {
            Url::parse(t).ok()
        } else {
            None
        };

        let input = arg.file_path.expect("No file path provided");
        //TODO: Expose configs
        let create_opt = Create {
            announce: primary_tracker,
            allowed_lints: vec![],
            announce_tiers: arg.trackers,
            comment: None,
            dht_nodes: vec![],
            dry_run: false,
            follow_symlinks: true,
            force: true,
            globs: vec![],
            include_hidden: false,
            include_junk: true,
            input_positional: Some(InputTarget::Path(input.into())),
            input_flag: None,
            print_magnet_link: true,
            md5sum: true,
            name: None,
            no_created_by: true,
            no_creation_date: true,
            open: false,
            sort_by: vec![],
            output: None,
            peers: vec![],
            piece_length: None,
            private: false,
            show: true,
            source: None,
            ignore: true,
            update_url: None,
        };

        create_opt
    }
}
