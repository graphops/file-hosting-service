# Publisher

This documentation provides a quick guide to publish verification files for indexed data files on IPFS. 

## To publish a Bundle

To start, you would need to provide several configurations

### Requirements

Publisher must have read access to all files contained in the Bundle. The publisher publish 1 Bundle at a time and is not responsible for hosting the file after publishing. The publisher should chunk all the files in the package and generate a hash for all the chunks. Then the publisher will build a hierarchy with the hashes. Currently, the Publisher simply put chunk hashes in a list for each individual file, publish individual file manifests, then they build a Bundle that contains a list of the file manifest addresses. 

> More exploration for hashing/packaging architecture


### CLI example

Publishing files stored in the local file system
```
$ file-exchange publisher \
  --bundle-name "blah" \
  --file-names example0017686312.dbin,example-create-17686085.dbin \
  --file-type flatfiles \
  --file-version 0.0.0 \
  --description "random flatfiles" \
  local-files --output-dir ./example-file/
```

Publishing files/objects stored in a remote s3 bucket
```
$ file-exchange publisher \
  --bundle-name "blah" \
  --file-names example0017686312.dbin,example-create-17686085.dbin \
  --file-type flatfiles \
  --file-version 0.0.0 \
  --description "random flatfiles" \
  object-storage --region ams3 \
   --bucket "contain-texture-dragon" \
   --access-key-id "DO0000000000000000" \
   --secret-key "secretttttttttt" \
   --endpoint "https://ams3.digitaloceanspaces.com" 
```
For more information 
```
$ file-exchange --help
```
