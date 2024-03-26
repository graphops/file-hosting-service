#[cfg(test)]
mod tests {
    use std::{process::Command, time::Duration};
    use tempfile::tempdir;
    use tokio::fs;

    use file_exchange::{
        config::{DownloaderArgs, LocalDirectory},
        download_client::Downloader,
        manifest::ipfs::IpfsClient,
        test_util::server_ready,
    };

    #[tokio::test]
    async fn test_file_transfer() {
        std::env::set_var("RUST_LOG", "off,file_exchange=trace,file_transfer=trace,file_service=trace,indexer_service=trace,indexer_common=trace");
        file_exchange::config::init_tracing("pretty").unwrap();

        let client = IpfsClient::new("https://ipfs.network.thegraph.com")
            .expect("Could not create client to thegraph IPFS gateway");
        let target_bundle = "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v".to_string();
        // 1. Setup server
        let mut server_process = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("file-service")
            .arg("--")
            .arg("--config")
            .arg("./tests/test0.toml")
            .spawn()
            .expect("Failed to start server");
        tracing::debug!("Wait 10 seconds");
        tokio::time::sleep(Duration::from_secs(3)).await;
        let _ = server_ready("http://localhost:5677").await;

        // 2. Setup downloader
        let temp_dir = tempdir().unwrap();
        let main_dir = temp_dir.path().to_path_buf();

        let downloader_args = DownloaderArgs {
            storage_method: file_exchange::config::StorageMethod::LocalFiles(LocalDirectory {
                main_dir: main_dir.to_str().unwrap().to_string(),
            }),
            ipfs_hash: target_bundle,
            indexer_endpoints: [
                "http://localhost:5679".to_string(),
                "http://localhost:5677".to_string(),
            ]
            .to_vec(),
            verifier: Some(String::from("0xfC24cE7a4428A6B89B52645243662A02BA734ECF")),
            mnemonic: None,
            free_query_auth_token: Some("Bearer free-token".to_string()),
            provider: None,
            provider_concurrency: 2,
            ..Default::default()
        };

        let downloader = Downloader::new(client, downloader_args).await;

        // 3. Perform the download
        let download_result = downloader.download_bundle().await;

        // 4. Validate the download
        tracing::info!(
            result = tracing::field::debug(&download_result),
            "Download result"
        );
        assert!(download_result.is_ok());
        // Further checks can be added to verify the contents of the downloaded files

        // 5. Cleanup
        fs::remove_dir_all(temp_dir).await.unwrap();
        let _ = server_process.kill();
    }
}
