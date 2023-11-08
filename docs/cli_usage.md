

### Publisher 
```

➜  subfile-exchange git:(main) ✗ cargo run -p subfile-cli publisher --read-dir ./example-file/ --subfile-name "blah" --file-name example0017686312.dbin --file-type nothing --file-version 0.0.0 --identifier example0017686312 --publisher-url "http://localhost:5678"

  2023-11-08T12:52:33.620845Z  INFO subfile_cli: Running cli, cli: Cli { role: Publisher(PublisherArgs { yaml_store: "./example-file/subfile.yaml", read_dir: "./example-file/", subfile_name: "blah", file_name: "example0017686312.dbin", file_type: "nothing", file_version: "0.0.0", identifier: "example0017686312", start_block: None, end_block: None, publisher_url: "http://localhost:5678" }), ipfs_gateway: "https://ipfs.network.thegraph.com", log_format: Pretty }
    at subfile-cli/src/main.rs:16

  2023-11-08T12:52:33.647797Z  INFO subfile_cli: Publisher request, config: PublisherArgs { yaml_store: "./example-file/subfile.yaml", read_dir: "./example-file/", subfile_name: "blah", file_name: "example0017686312.dbin", file_type: "nothing", file_version: "0.0.0", identifier: "example0017686312", start_block: None, end_block: None, publisher_url: "http://localhost:5678" }
    at subfile-cli/src/main.rs:44

  2023-11-08T12:52:33.648150Z DEBUG subfile_cli::file_hasher: Start chunking, path: "./example-file//example0017686312.dbin"
    at subfile-cli/src/file_hasher.rs:38

  2023-11-08T12:52:33.652072Z DEBUG subfile_cli::file_hasher: Chunked, total_bytes: 1508787, num_chunks: 2
    at subfile-cli/src/file_hasher.rs:51

  2023-11-08T12:52:33.652184Z DEBUG subfile_cli::file_hasher: File chunked, total_bytes: 1508787
    at subfile-cli/src/file_hasher.rs:95

  2023-11-08T12:52:33.742251Z DEBUG subfile_cli::file_hasher: Chunk hash, hash: [202, 227, 94, 140, 125, 254, 145, 68, 217, 119, 24, 19, 178, 40, 120, 144, 232, 4, 19, 190, 183, 234, 100, 36, 76, 83, 65, 117, 96, 23, 26, 146]
    at subfile-cli/src/file_hasher.rs:26

  2023-11-08T12:52:33.742375Z DEBUG subfile_cli::file_hasher: Chunk hash, hash_str: "yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI="
    at subfile-cli/src/file_hasher.rs:28

  2023-11-08T12:52:33.781561Z DEBUG subfile_cli::file_hasher: Chunk hash, hash: [191, 27, 225, 27, 22, 159, 177, 171, 252, 216, 173, 16, 107, 156, 16, 160, 148, 174, 246, 237, 228, 234, 254, 114, 77, 246, 129, 45, 195, 205, 203, 231]
    at subfile-cli/src/file_hasher.rs:26

  2023-11-08T12:52:33.781585Z DEBUG subfile_cli::file_hasher: Chunk hash, hash_str: "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="
    at subfile-cli/src/file_hasher.rs:28

  2023-11-08T12:52:33.781709Z TRACE subfile_cli::file_hasher: Created chunk file, file: ChunkFile { file_name: "example0017686312.dbin", total_bytes: 1508787, chunk_size: 1048576, chunk_hashes: ["yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI=", "vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c="] }
    at subfile-cli/src/file_hasher.rs:113

  2023-11-08T12:52:33.782752Z DEBUG subfile_cli::publisher: chunk file yaml bytes, yaml: [102, 105, 108, 101, 95, 110, 97, 109, 101, 58, 32, 101, 120, 97, 109, 112, 108, 101, 48, 48, 49, 55, 54, 56, 54, 51, 49, 50, 46, 100, 98, 105, 110, 10, 116, 111, 116, 97, 108, 95, 98, 121, 116, 101, 115, 58, 32, 49, 53, 48, 56, 55, 56, 55, 10, 99, 104, 117, 110, 107, 95, 115, 105, 122, 101, 58, 32, 49, 48, 52, 56, 53, 55, 54, 10, 99, 104, 117, 110, 107, 95, 104, 97, 115, 104, 101, 115, 58, 10, 45, 32, 121, 117, 78, 101, 106, 72, 51, 43, 107, 85, 84, 90, 100, 120, 103, 84, 115, 105, 104, 52, 107, 79, 103, 69, 69, 55, 54, 51, 54, 109, 81, 107, 84, 70, 78, 66, 100, 87, 65, 88, 71, 112, 73, 61, 10, 45, 32, 118, 120, 118, 104, 71, 120, 97, 102, 115, 97, 118, 56, 50, 75, 48, 81, 97, 53, 119, 81, 111, 74, 83, 117, 57, 117, 51, 107, 54, 118, 53, 121, 84, 102, 97, 66, 76, 99, 80, 78, 121, 43, 99, 61, 10]
    at subfile-cli/src/publisher.rs:41

  2023-11-08T12:52:34.835804Z DEBUG subfile_cli::publisher: Added yaml file to IPFS, added: AddResponse { name: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ", hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ", size: "194" }
    at subfile-cli/src/publisher.rs:44

Published file to IPFS: AddResponse {
    name: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ",
    hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ",
    size: "194",
}
  2023-11-08T12:52:34.836098Z DEBUG subfile_cli::publisher: Published, hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ"
    at subfile-cli/src/publisher.rs:100

Published subfile manifest to IPFS with hash: QmeuvDXTWHDye4eBQhQR31MFTHdXHWedZchYJ31Qij1ZbX
  2023-11-08T12:52:35.362648Z  INFO subfile_cli: Published, result: "QmeuvDXTWHDye4eBQhQR31MFTHdXHWedZchYJ31Qij1ZbX"
    at subfile-cli/src/main.rs:50


```



### Server

To server 1 file


```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-cli server \
    --host 0.0.0.0 \
    --port 5678


``````


Corresponding client

```
➜  ~ curl -H "Range: bytes=0-102000003" http://localhost:5678/subfiles/id/QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX
> range out of bound

➜  ~ curl -H "Range: bytes=0-1023" http://localhost:5678/subfiles/id/QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX --output -  
> [partial file...]

➜  ~ curl http://localhost:5678/subfiles/id/QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX 
> [whole file...]
```



### Downloader

```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-cli downloader --ipfs-hash QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ --gateway-url http://localhost:5678/subfiles/id/


  2023-11-08T13:24:01.154561Z  INFO subfile_cli: Running cli, cli: Cli { role: Downloader(DownloaderArgs { ipfs_hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ", gateway_url: "http://localhost:5678/subfiles/id/", output_dir: "./example-download" }), ipfs_gateway: "https://ipfs.network.thegraph.com", log_format: Pretty }
    at subfile-cli/src/main.rs:16

  2023-11-08T13:24:01.193448Z  INFO subfile_cli: Downloader request, config: DownloaderArgs { ipfs_hash: "QmSy2UtZNJbwWFED6CroKzRmMz43WjrN8Y1Bns1EFqjeKJ", gateway_url: "http://localhost:5678/subfiles/id/", output_dir: "./example-download" }
    at subfile-cli/src/main.rs:26

  2023-11-08T13:24:02.087272Z  INFO subfile_cli::subfile_reader: Read file content, content: Mapping {"file_name": String("example0017686312.dbin"), "total_bytes": Number(1508787), "chunk_size": Number(1048576), "chunk_hashes": Sequence [String("yuNejH3+kUTZdxgTsih4kOgEE7636mQkTFNBdWAXGpI="), String("vxvhGxafsav82K0Qa5wQoJSu9u3k6v5yTfaBLcPNy+c=")]}
    at subfile-cli/src/subfile_reader.rs:53

  2023-11-08T13:24:02.089457Z TRACE subfile_cli::subfile_client: Download chunk index, i: 0
    at subfile-cli/src/subfile_client/mod.rs:83

  2023-11-08T13:24:02.089920Z TRACE subfile_cli::subfile_client: Download chunk index, i: 1
    at subfile-cli/src/subfile_client/mod.rs:83

  2023-11-08T13:24:02.140043Z  INFO subfile_cli::subfile_client: chunk_download_result: ()
    at subfile-cli/src/subfile_client/mod.rs:101

  2023-11-08T13:24:02.140103Z  INFO subfile_cli::subfile_client: chunk_download_result: ()
    at subfile-cli/src/subfile_client/mod.rs:101

Download result: Ok(
    (),
)
```

