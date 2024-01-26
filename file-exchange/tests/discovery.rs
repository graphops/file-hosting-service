#[cfg(test)]
mod tests {
    use std::{process::Command, time::Duration};

    use file_exchange::{
        discover::{unavailable_files, FileAvailbilityMap, Finder, IndexerEndpoint},
        manifest::ipfs::IpfsClient,
        test_util::server_ready,
    };

    // TODO: set up test environment
    #[tokio::test]
    // #[ignore]
    async fn test_discovery() {
        // 0. Basic setup; const
        std::env::set_var("RUST_LOG", "off,file_exchange=debug,file_transfer=trace");
        file_exchange::config::init_tracing("pretty").unwrap();

        let server_0 = "http://0.0.0.0:5677";
        let server_1 = "http://0.0.0.0:5679";

        let file_manifest_hash_a = "QmeKabcCQBtgU6QjM3rp3w6pDHFW4r54ee89nGdhuyDuhi".to_string();
        let file_manifest_hash_b = "QmeE38uPSqT5XuHfM8X2JZAYgDCEwmDyMYULmZaRnNqPCj".to_string();
        let file_manifest_hash_c = "QmWs8dkshZ7abxFYQ3h9ie1Em7SqzAkwtVJXaBapwEWqR9".to_string();

        let bundle_hash_0 = "QmeaPp764FjQjPB66M9ijmQKmLhwBpHQhA7dEbH2FA1j3v".to_string(); // files: A,B,C
        let bundle_hash_1 = "QmVPPWWaraEvoc4LCrYXtMbL13WPNbnuXV2yo7W8zexFGq".to_string(); // files: A
        let bundle_hash_2 = "QmeD3dRVV6Gs84TRwiNj3tLt9mBEMVqy3GoWm7WN8oDzGz".to_string(); // files: B,C
        let bundle_hash_3 = "QmTSwj1BGkkmVSnhw6uEGkcxGZvP5nq4pDhzHjwJvsQC2Z".to_string(); // files: B

        let indexer_0: IndexerEndpoint = (
            "0xead22a75679608952db6e85537fbfdca02dae9cb".to_string(),
            server_0.to_string(),
        );
        let indexer_1: IndexerEndpoint = (
            "0x19804e50af1b72db4ce22a3c028e80c78d75af62".to_string(),
            "http://0.0.0.0:5679".to_string(),
        );

        // 1. Setup servers 0 and 1
        let mut server_process_0 = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("file-service")
            .arg("--")
            .arg("--config")
            .arg("./../test.toml")
            .spawn()
            .expect("Failed to start server");

        let mut server_process_1 = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("file-service")
            .arg("--")
            .arg("--config")
            .arg("./../test2.toml")
            .spawn()
            .expect("Failed to start server");

        tracing::debug!("Server initializing, wait 10 seconds...");
        tokio::time::sleep(Duration::from_secs(10)).await;
        let _ = server_ready(server_0).await;
        let _ = server_ready(server_1).await;

        // 2. Setup finder
        let client = IpfsClient::new("https://ipfs.network.thegraph.com")
            .expect("Could not create client to thegraph IPFS gateway");
        let finder = Finder::new(client);

        // 3. Find various combinations of bundles
        // 3.1 find bundle_0 with server 0 and 1, get server 0
        let endpoints = finder
            .bundle_availabilities(
                &bundle_hash_0,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await;
        println!("endpoiunt: {:#?}", endpoints);
        assert!(endpoints.len() == 1);
        assert!(endpoints.first().unwrap().0 == "0xead22a75679608952db6e85537fbfdca02dae9cb");
        assert!(endpoints.first().unwrap().1 == server_0);

        // 3.2 find bundle_1 with server 0 and 1, get server 1
        let endpoints = finder
            .bundle_availabilities(
                &bundle_hash_1,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await;
        assert!(endpoints.len() == 1);
        assert!(endpoints.first().unwrap().0 == "0x19804e50af1b72db4ce22a3c028e80c78d75af62");
        assert!(endpoints.first().unwrap().1 == server_1);

        // 3.3 find bundle_0 with sieved availability
        let map = finder
            .file_discovery(
                &bundle_hash_0,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await
            .unwrap();
        assert!(map.lock().await.len() == 3);
        assert!(
            matched(
                &map,
                &file_manifest_hash_a,
                &indexer_0,
                &vec![&bundle_hash_0]
            )
            .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_b,
                &indexer_0,
                &vec![&bundle_hash_0]
            )
            .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_c,
                &indexer_0,
                &vec![&bundle_hash_0]
            )
            .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_a,
                &indexer_1,
                &vec![&bundle_hash_1]
            )
            .await
        );
        // update innermost vec to be a hashset to avoid ordering problem
        assert!(
            matched(
                &map,
                &file_manifest_hash_b,
                &indexer_1,
                &vec![&bundle_hash_3, &bundle_hash_2]
            )
            .await
                || matched(
                    &map,
                    &file_manifest_hash_b,
                    &indexer_1,
                    &vec![&bundle_hash_2, &bundle_hash_3]
                )
                .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_c,
                &indexer_1,
                &vec![&bundle_hash_2]
            )
            .await
        );

        // 3.4 find bundle_1 with sieved availability, get
        let map = finder
            .file_discovery(
                &bundle_hash_1,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await
            .unwrap();
        assert!(map.lock().await.len() == 1);
        assert!(
            matched(
                &map,
                &file_manifest_hash_a,
                &indexer_0,
                &vec![&bundle_hash_0]
            )
            .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_a,
                &indexer_1,
                &vec![&bundle_hash_1]
            )
            .await
        );

        // 3.5 find bundle_2 with sieved availability, get both 0 and 1
        let map = finder
            .file_discovery(
                &bundle_hash_2,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await
            .unwrap();
        assert!(map.lock().await.len() == 2);
        assert!(
            matched(
                &map,
                &file_manifest_hash_b,
                &indexer_0,
                &vec![&bundle_hash_0]
            )
            .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_c,
                &indexer_0,
                &vec![&bundle_hash_0]
            )
            .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_b,
                &indexer_1,
                &vec![&bundle_hash_3, &bundle_hash_2]
            )
            .await
                || matched(
                    &map,
                    &file_manifest_hash_b,
                    &indexer_1,
                    &vec![&bundle_hash_2, &bundle_hash_3]
                )
                .await
        );
        assert!(
            matched(
                &map,
                &file_manifest_hash_c,
                &indexer_1,
                &vec![&bundle_hash_2]
            )
            .await
        );

        // 3.6 large files, not available on neither
        let large_bundle_hash = "QmPexYQsJKyhL867xRaGS2kciNDwggCk7pgUxrNoPQSuPL"; // contains File A,B,C,D,E
        let endpoints = finder
            .bundle_availabilities(
                large_bundle_hash,
                &[server_0.to_string(), server_1.to_string()],
            )
            .await;
        assert!(endpoints.is_empty());
        let map = finder
            .file_discovery(
                large_bundle_hash,
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
        file_manifest: &str,
        endpoint: &IndexerEndpoint,
        bundle_hashes: &Vec<&str>,
    ) -> bool {
        let map = file_map.lock().await;
        // Check if the key exists in the outer HashMap
        let file_manifest_map = map.get(file_manifest).unwrap();
        let inner_map = file_manifest_map.lock().await;

        // Check if the endpoint exists in the inner HashMap
        let bundles = inner_map.get(endpoint).unwrap();
        bundle_hashes == bundles
    }
}
