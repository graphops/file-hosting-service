// #![deny(warnings)]
// use hyper;

// use std::env;
// use std::io::{self, Write};
// use hyper::Client;
// use hyper::header::{Connection, Range, ByteRangeSpec};

use anyhow::anyhow;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use http::header::{CONTENT_RANGE, RANGE};

use crate::config::DownloaderArgs;
use crate::ipfs::IpfsClient;

pub struct SubfileDownloader {
    ipfs_client: IpfsClient,
    http_client: reqwest::Client,
    ipfs_hash: String,
    //TODO: currently direct ping to server_url -> decentralize to gateway_url
    server_url: String,
    output_dir: String,
    // Other fields as needed
}

impl SubfileDownloader {
    pub fn new(ipfs_client: IpfsClient, args: DownloaderArgs) -> Self {
        SubfileDownloader {
            ipfs_client,
            //TODO: consider if more advanced config like 
            // a proxy should be used for the client
            http_client: reqwest::Client::new(),
            ipfs_hash: args.ipfs_hash,
            //TODO: change for discovery
            server_url: args.gateway_url,
            output_dir: args.output_dir,
        }
    }

    pub async fn check_availability(&self) -> Result<(), anyhow::Error> {
        todo!()
    }

    pub async fn send_request(&self) -> Result<(), anyhow::Error> {
        // Assuming the URL is something like "http://localhost:5678/subfiles/id/"
        let url = format!("{}{}", self.server_url, self.ipfs_hash);

        // For example, to request the first 1024 bytes
        // The client should be smart enough to take care of proper chunking through subfile metadata
        let range = "bytes=1023-2023";

        let response = self
            .http_client
            .get(&url)
            .header(RANGE, range)
            .send()
            .await?;

        // Check if the server supports range requests
        if response.status().is_success() && response.headers().contains_key(CONTENT_RANGE) {
            let content = response.bytes().await?;
            let output_path = PathBuf::from(&self.output_dir).join(&self.ipfs_hash);
            let mut file = File::create(output_path)?;
            file.write_all(&content)?;
        } else {
            eprintln!("Server does not support range requests or the request failed.");
            return Err(anyhow!("Range request failed"));
        }

        Ok(())
    }
}

