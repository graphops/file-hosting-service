


### CLI Usage

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