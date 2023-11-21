## To publish a subfile

To start, you would need to provide several configurations

### Requirements

Publisher must have read access to all files contained in the package. The publisher publish 1 subfile at a time and is not responsible for hosting the file after publishing. The publisher should chunk all the files in the package and generate a hash for all the chunks. Then the publisher will build a hierarchy with the hashes. Currently, publisher simply put chunk hashes in a list for each individual files, publish individual chunk files, then they build a subfile that contains a list of the chunk file addresses. 

> More exploration for hashing/packaging architecture


### CLI example
```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-exchange publisher \
  --read-dir ./example-file/ \
  --subfile-name "blah" \
  --file-names example0017686312.dbin,example-create-17686085.dbin \
  --file-type nothing \
  --file-version 0.0.0 \
  --identifier example0017686312 \
  --publisher-url "http://localhost:5678" 
```




### Configuration matrix

| Environment Variable                          | CLI Argument                    | Value                                            |
| --------------------------------------------- | ------------------------------- | ------------------------------------------------ |
| `READ_DIR`             | `--read-dir`             | Read directory for the files to publish               |
| `SUBFILE_NAME`             | `--subfile-name`             | Give a name to the subfile (Q: removable as it can cause unnecessary change in subfile hash)               |
| `FILE_TYPE`             | `--file-type`             | flatfile, snapshot, ...               |
| `FILE_VERSION`             | `--file-version`             | Subfile version               |
| `FILE_NAMES`             | `--file-names`             | name of the files to include in the package               |
| `IDENTIFIER`             | `--identifier`             | Useful for deployment specific files                |
| `PUBLISHER_URL`             | `--publisher-url`             | include in subfile manifest for self advertisement               |

### To add
| Environment Variable                          | CLI Argument                    | Value                                            |
| --------------------------------------------- | ------------------------------- | ------------------------------------------------ |
| `SUBFILE_SERVICE_MNEMONIC`                    | `--mnemonic`                    | Ethereum mnemonic for connecting to a wallet for publishing           |
| `SUBFILE_SERVICE_ETHEREUM_NETWORK`            | `--ethereum-network`            | `mainnet`, `goerli`, `arbitrum-one`, `arbigrum-goerli`                              |

Consider access such as to postgres or files requiring authentication
