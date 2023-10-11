

### Requirement

Set up a IPFS gateway or use fallback at thegraph
```
sudo ipfs daemon
```


### Essential structs
- subfile struct
- ipfs client
- yaml parser and builder
- leecher essentials
- tracker essentials
- seeder essentials

### Leecher

Essential
- [ ] Use IPFS client to cat the file (in bytes),
- [ ] Parse bytes into a subfile yaml file, fit into a subfile struct, 
- [ ] Grab the magnet link from subfile.yaml and start torrent leeching

Optional
- [ ] Validate IPFS against extra input to make sure it is the target file

### CLI Usage

Set RUST_LOG envvar
Provide preferred ipfs gaetway, fallback with thegraph ipfs gateway
Set Log format, fallback with pretty

```
> cargo run
Subfile data service - hackathon

Usage: subfile-exchange <COMMAND>

Commands:
  leecher  
  seeder   
  tracker  
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```


```
> cargo run leecher
error: the following required arguments were not provided:
  --ipfs-hash <IPFS_HASH>

Usage: subfile-exchange leecher --ipfs-hash <IPFS_HASH>
```

More usage stuff

```
Usage: subfile-exchange tracker --server-host <SERVER_HOST> --server-port <SERVER_PORT>
Usage: subfile-exchange seeder --file-path <FILE_PATH> --file-type <FILE_TYPE> --identifier <IDENTIFIER>
```



