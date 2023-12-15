#[cfg(test)]
mod tests {
    use std::{process::Command, time::Duration};

    use subfile_exchange::{
        subfile::ipfs::IpfsClient,
        subfile_finder::{unavailable_files, FileAvailbilityMap, IndexerEndpoint, SubfileFinder},
        test_util::server_ready,
    };

    #[tokio::test]
    async fn test_discovery() {
        // 0. Basic setup; const
        std::env::set_var("RUST_LOG", "off,subfile_exchange=debug,file_transfer=trace");
        subfile_exchange::config::init_tracing(String::from("pretty")).unwrap();

        let chunk_file_hash_a = "QmeKabcCQBtgU6QjM3rp3w6pDHFW4r54ee89nGdhuyDuhi".to_string();
        let chunk_file_hash_b = "QmeE38uPSqT5XuHfM8X2JZAYgDCEwmDyMYULmZaRnNqPCj".to_string();
        let chunk_file_hash_c = "QmWs8dkshZ7abxFYQ3h9ie1Em7SqzAkwtVJXaBapwEWqR9".to_string();

        let subfile_hash_0 = "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v".to_string(); // files: A,B,C
        let subfile_hash_1 = "QmVPPWWaraEvoc4LCrYXtMbL13WPNbnuXV2yo7W8zexFGq".to_string(); // files: A
        let subfile_hash_2 = "QmeD3dRVV6Gs84TRwiNj3tLt9mBEMVqy3GoWm7WN8oDzGz".to_string(); // files: B,C
        let subfile_hash_3 = "QmTSwj1BGkkmVSnhw6uEGkcxGZvP5nq4pDhzHjwJvsQC2Z".to_string(); // files: B

        let indexer_0: IndexerEndpoint = (
            "0xead22a75679608952db6e85537fbfdca02dae9cb".to_string(),
            "http://0.0.0.0:5678".to_string(),
        );
        let indexer_1: IndexerEndpoint = (
            "0x19804e50af1b72db4ce22a3c028e80c78d75af62".to_string(),
            "http://0.0.0.0:5679".to_string(),
        );

        // 1. Setup servers 0 and 1
        let mut server_process_0 = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("subfile-exchange")
            .arg("server")
            .arg("--mnemonic")
            .arg("sheriff obscure trick beauty army fat wink legal flee leader section suit")
            .arg("--subfiles")
            .arg(format!("{}:./../example-file/", subfile_hash_0))
            .spawn()
            .expect("Failed to start server");

        let mut server_process_1 = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("subfile-exchange")
            .arg("server")
            .arg("--mnemonic")
            .arg("ice palace drill gadget biology glow tray equip heavy wolf toddler menu")
            .arg("--host")
            .arg("0.0.0.0")
            .arg("--port")
            .arg("5679")
            .arg("--subfiles")
            .arg(format!(
                "{}:./../example-file/,{}:./../example-file/,{}:./../example-file/",
                subfile_hash_1, subfile_hash_2, subfile_hash_3
            ))
            .spawn()
            .expect("Failed to start server");

        tracing::debug!("Server initializing, wait 10 seconds...");
        tokio::time::sleep(Duration::from_secs(10)).await;
        let server_0 = "http://0.0.0.0:5678";
        let server_1 = "http://0.0.0.0:5679";
        let _ = server_ready(server_0).await;
        let _ = server_ready(server_1).await;

        // 2. Setup finder
        let client = IpfsClient::new("https://ipfs.network.thegraph.com")
            .expect("Could not create client to thegraph IPFS gateway");
        let finder = SubfileFinder::new(client);

        // 3. Find various combinations of subfiles
        // 3.1 find subfile_0 with server 0 and 1, get server 0
        let endpoints = finder
            .subfile_availabilities(
                &subfile_hash_0,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await;
        assert!(endpoints.len() == 1);
        assert!(endpoints.first().unwrap().0 == "0xead22a75679608952db6e85537fbfdca02dae9cb");
        assert!(endpoints.first().unwrap().1 == "http://0.0.0.0:5678");

        // 3.2 find subfile_1 with server 0 and 1, get server 1
        let endpoints = finder
            .subfile_availabilities(
                &subfile_hash_1,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await;
        assert!(endpoints.len() == 1);
        assert!(endpoints.first().unwrap().0 == "0x19804e50af1b72db4ce22a3c028e80c78d75af62");
        assert!(endpoints.first().unwrap().1 == "http://0.0.0.0:5679");

        // 3.3 find subfile_0 with sieved availability
        let map = finder
            .file_discovery(
                &subfile_hash_0,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await
            .unwrap();
        assert!(map.lock().await.len() == 3);
        assert!(matched(&map, &chunk_file_hash_a, &indexer_0, &vec![&subfile_hash_0]).await);
        assert!(matched(&map, &chunk_file_hash_b, &indexer_0, &vec![&subfile_hash_0]).await);
        assert!(matched(&map, &chunk_file_hash_c, &indexer_0, &vec![&subfile_hash_0]).await);
        assert!(matched(&map, &chunk_file_hash_a, &indexer_1, &vec![&subfile_hash_1]).await);
        // update innermost vec to be a hashset to avoid ordering problem
        assert!(
            matched(
                &map,
                &chunk_file_hash_b,
                &indexer_1,
                &vec![&subfile_hash_3, &subfile_hash_2]
            )
            .await
                || matched(
                    &map,
                    &chunk_file_hash_b,
                    &indexer_1,
                    &vec![&subfile_hash_2, &subfile_hash_3]
                )
                .await
        );
        assert!(matched(&map, &chunk_file_hash_c, &indexer_1, &vec![&subfile_hash_2]).await);

        // 3.4 find subfile_1 with sieved availability, get
        let map = finder
            .file_discovery(
                &subfile_hash_1,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await
            .unwrap();
        assert!(map.lock().await.len() == 1);
        assert!(matched(&map, &chunk_file_hash_a, &indexer_0, &vec![&subfile_hash_0]).await);
        assert!(matched(&map, &chunk_file_hash_a, &indexer_1, &vec![&subfile_hash_1]).await);

        // 3.5 find subfile_2 with sieved availability, get both 0 and 1
        let map = finder
            .file_discovery(
                &subfile_hash_2,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await
            .unwrap();
        assert!(map.lock().await.len() == 2);
        assert!(matched(&map, &chunk_file_hash_b, &indexer_0, &vec![&subfile_hash_0]).await);
        assert!(matched(&map, &chunk_file_hash_c, &indexer_0, &vec![&subfile_hash_0]).await);
        assert!(
            matched(
                &map,
                &chunk_file_hash_b,
                &indexer_1,
                &vec![&subfile_hash_3, &subfile_hash_2]
            )
            .await
                || matched(
                    &map,
                    &chunk_file_hash_b,
                    &indexer_1,
                    &vec![&subfile_hash_2, &subfile_hash_3]
                )
                .await
        );
        assert!(matched(&map, &chunk_file_hash_c, &indexer_1, &vec![&subfile_hash_2]).await);

        // 3.6 large files, not available on neither
        let large_subfile_hash = "QmPexYQsJKyhL867xRaGS2kciNDwggCk7pgUxrNoPQSuPL"; // contains File A,B,C,D,E
        let endpoints = finder
            .subfile_availabilities(
                large_subfile_hash,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await;
        assert!(endpoints.is_empty());
        let map = finder
            .file_discovery(
                large_subfile_hash,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await
            .unwrap();
        let unavailable_files = unavailable_files(&map).await;
        assert!(unavailable_files.len() == 2);
        assert!(unavailable_files.contains(&String::from(
            "QmSydRNSzjozo5d7W4AyCK8BkgfpEU8KQp9kvSHzf2Ch4g"
        )));
        assert!(unavailable_files.contains(&String::from(
            "QmSuyvzDpuDBoka2rCimRXPmX2icL7Vu6RUxoFWFQD7YBb"
        )));

        // 4. Cleanup
        let _ = server_process_0.kill();
        let _ = server_process_1.kill();
    }

    async fn matched(
        file_map: &FileAvailbilityMap,
        chunk_file: &str,
        endpoint: &IndexerEndpoint,
        subfile_hashes: &Vec<&str>,
    ) -> bool {
        let map = file_map.lock().await;
        // Check if the key exists in the outer HashMap
        let chunk_file_map = map.get(chunk_file).unwrap();
        let inner_map = chunk_file_map.lock().await;

        // Check if the endpoint exists in the inner HashMap
        let subfiles = inner_map.get(endpoint).unwrap();
        subfile_hashes == subfiles
    }
}
