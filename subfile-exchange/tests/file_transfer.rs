#[cfg(test)]
mod tests {
    use http::StatusCode;
    use std::{process::Command, time::Duration};
    use tempfile::tempdir;
    use tokio::fs;

    use subfile_exchange::{
        config::DownloaderArgs, ipfs::IpfsClient, subfile_client::SubfileDownloader,
    };

    #[tokio::test]
    async fn test_file_transfer() {
        std::env::set_var("RUST_LOG", "off,subfile_exchange=info,file_transfer=trace");
        subfile_exchange::config::init_tracing(String::from("pretty")).unwrap();

        let client = if let Ok(client) = IpfsClient::new("https://ipfs.network.thegraph.com") {
            client
        } else {
            IpfsClient::localhost()
        };

        // 1. Setup server
        let mut server_process = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("subfile-exchange")
            .arg("server")
            .arg("--mnemonic")
            .arg("sheriff obscure trick beauty army fat wink legal flee leader section suit")
            .arg("--subfiles")
            .arg("QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ:./../example-file/")
            .spawn()
            .expect("Failed to start server");
        tracing::debug!("Wait 10 seconds");
        tokio::time::sleep(Duration::from_secs(10)).await;
        let _ = server_ready("http://0.0.0.0:5678/status").await;

        // 2. Setup downloader
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        let downloader_args = DownloaderArgs {
            output_dir: output_dir.to_str().unwrap().to_string(),
            ipfs_hash: "QmUqx9seQqAuCRi3uEPfa1rcS61rKhM7JxtraL81jvY6dZ".to_string(),
            indexer_endpoints: [
                "http://localhost:5678".to_string(),
                "http://localhost:5677".to_string(),
            ]
            .to_vec(),
            free_query_auth_token: Some("Bearer free-token".to_string()),
            ..Default::default()
        };

        let downloader = SubfileDownloader::new(client, downloader_args);

        // 3. Perform the download
        let download_result = downloader.download_subfile().await;

        // 4. Validate the download
        assert!(download_result.is_ok());
        // Further checks can be added to verify the contents of the downloaded files

        // 5. Cleanup
        fs::remove_dir_all(temp_dir).await.unwrap();
        let _ = server_process.kill();
    }

    async fn server_ready(url: &str) -> Result<(), anyhow::Error> {
        loop {
            match reqwest::get(url).await {
                Ok(response) => {
                    if response.status() == StatusCode::OK {
                        tracing::trace!("Server is healthy!");
                        return Ok(());
                    } else {
                        tracing::trace!("Server responded with status: {}", response.status());
                    }
                }
                Err(e) => {
                    tracing::trace!("Failed to connect to server: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}