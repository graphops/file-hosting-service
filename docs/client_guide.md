## Request service

### Requirements

To start, you would need to determine the subfile that contains the dataset you desire. This may mean looking at subfile manifests or in the future a tool that matches subfiles by some critieria. 

After determining the subfile you want to buy, you also need to supply a local path for writing the subfile corresponding files, a wallet for payments or a free query auth token, and a list of indexer endpoints (this should be handled by gateway or a scraping client).

### CLI example
```
➜  subfile-exchange git:(main) ✗ cargo run -p subfile-exchange downloader \
   --ipfs-hash QmakV6VEwnydfe7PXFR3TRxHbhVm7mQRXqVHdsizhTRrGw \
   --indexer-endpoints http://localhost:5678 \
   --free-query-auth-token 'Bearer imfreeee'
```

### To add

`--mnemonic` for billing
