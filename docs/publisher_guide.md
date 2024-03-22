# Publisher

This documentation provides a quick guide to publish verification files for indexed data files on IPFS. 

## To publish a Bundle

To start, you would need to provide several configurations

### Requirements

Publisher must have read access to all files contained in the Bundle. The publisher publish 1 Bundle at a time and is not responsible for hosting the file after publishing. 

**Expectations**
1. For each file in the bundle, the publisher chunk the files into specified sizes and generate a hash for all the chunks. 
2. The publisher creates a file manifest containing information on the total number of bytes, chunk sizes, and an ordered list of chunk hashes. 
3. The publisher publishs individual file manifests, 
4. The publisher creates a bundle manifest containing information on the file names, file manfiest addresses, file types, and other meta descriptions.


### CLI usage

The publisher must provide a name for the bundle, filenames, file type, version, and path for read-access of the files.

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
