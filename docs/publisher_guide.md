# Publisher

This documentation provides a quick guide to publish verification files for indexed data files on IPFS. 

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
  --file-type flatfiles \
  --file-version 0.0.0 \
  --description "random flatfiles" \
  --chain-id 1
```

For more information 
```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-exchange --help
```
