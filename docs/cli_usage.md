

Publisher 
```


➜  subfile-exchange git:(main) ✗ cargo run -p subfile-cli publisher \
    --file-path ./example-file/example0017686312.dbin \
    --file-type nothing \
    --file-version 0.0.0 \
    --identifier example0017686312 \
    --publisher-url "http://localhost:5678"

  2023-11-07T17:16:05.687955Z  INFO subfile_cli: Running cli, cli: Cli { role: Publisher(PublisherArgs { yaml_store: "./example-file/subfile.yaml", file_path: "./example-file/example0017686312.dbin", name: None, file_type: "nothing", file_version: "0.0.0", identifier: "example0017686312", start_block: None, end_block: None, publisher_url: "http://localhost:5678" }), ipfs_gateway: "https://ipfs.network.thegraph.com", log_format: Pretty }
    at subfile-cli/src/main.rs:16

  2023-11-07T17:16:05.699977Z  INFO subfile_cli: Publisher request, config: PublisherArgs { yaml_store: "./example-file/subfile.yaml", file_path: "./example-file/example0017686312.dbin", name: None, file_type: "nothing", file_version: "0.0.0", identifier: "example0017686312", start_block: None, end_block: None, publisher_url: "http://localhost:5678" }
    at subfile-cli/src/main.rs:56

  2023-11-07T17:16:14.207580Z  INFO subfile_cli::publisher: Added yaml file to IPFS, added: AddResponse { name: "Qmc8busNmyhC9isrsqy9p8WSjSo1S67fBZhbXoxZaDqDJC", hash: "Qmc8busNmyhC9isrsqy9p8WSjSo1S67fBZhbXoxZaDqDJC", size: "3018486" }
    at subfile-cli/src/publisher.rs:40

Published file to IPFS: AddResponse {
    name: "Qmc8busNmyhC9isrsqy9p8WSjSo1S67fBZhbXoxZaDqDJC",
    hash: "Qmc8busNmyhC9isrsqy9p8WSjSo1S67fBZhbXoxZaDqDJC",
    size: "3018486",
}
Published subfile manifest to IPFS with hash: QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX
  2023-11-07T17:16:14.792428Z  INFO subfile_cli: Published, result: "QmPUsWnSoosNmM2uaKQwZRfEDJpxVciV2UjwycBdv7HsoX"
    at subfile-cli/src/main.rs:62



```



Server


```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-cli server \
    --host 0.0.0.0 \
    --port 5678


``````