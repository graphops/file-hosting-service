use bytes::Bytes;
use futures::Stream;
use http::header::CONTENT_LENGTH;
use http::Uri;
use reqwest::multipart;
use serde::Deserialize;

use std::time::Duration;
use std::{str::FromStr, sync::Arc};

/// Reference type, clones will share the connection pool.
#[derive(Clone, Debug)]
pub struct IpfsClient {
    base: Arc<Uri>,
    // reqwest::Client doesn't need to be `Arc` because it has one internally
    // already.
    client: reqwest::Client,
}

impl IpfsClient {
    pub fn new(base: &str) -> Result<Self, http::uri::InvalidUri> {
        Ok(IpfsClient {
            client: reqwest::Client::new(),
            base: Arc::new(Uri::from_str(base)?),
        })
    }

    pub fn localhost() -> Self {
        IpfsClient {
            client: reqwest::Client::new(),
            base: Arc::new(Uri::from_str("http://localhost:5001").unwrap()),
        }
    }

    /// Download the entire contents.
    pub async fn cat_all(&self, cid: &str, timeout: Duration) -> Result<Bytes, reqwest::Error> {
        self.call(self.url("cat", cid), None, Some(timeout))
            .await?
            .bytes()
            .await
    }

    pub async fn cat(
        &self,
        cid: &str,
        timeout: Option<Duration>,
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, reqwest::Error> {
        Ok(self
            .call(self.url("cat", cid), None, timeout)
            .await?
            .bytes_stream())
    }

    pub async fn test(&self) -> Result<(), reqwest::Error> {
        self.call(format!("{}api/v0/version", self.base), None, None)
            .await
            .map(|_| ())
    }

    pub async fn add(&self, data: Vec<u8>) -> Result<AddResponse, reqwest::Error> {
        let form = multipart::Form::new().part("path", multipart::Part::bytes(data));

        self.call(format!("{}api/v0/add", self.base), Some(form), None)
            .await?
            .json()
            .await
    }

    fn url(&self, route: &str, arg: &str) -> String {
        // URL security: We control the base and the route, user-supplied input goes only into the
        // query parameters.
        format!("{}api/v0/{}?arg={}", self.base, route, arg)
    }

    async fn call(
        &self,
        url: String,
        form: Option<multipart::Form>,
        timeout: Option<Duration>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let mut req = self.client.post(&url);
        if let Some(form) = form {
            req = req.multipart(form);
        } else {
            // Some servers require `content-length` even for an empty body.
            req = req.header(CONTENT_LENGTH, 0);
        }

        if let Some(timeout) = timeout {
            req = req.timeout(timeout)
        }

        req.send()
            .await
            .map(|res| res.error_for_status())
            .and_then(|x| x)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddResponse {
    pub name: String,
    pub hash: String,
    pub size: String,
}

pub fn create_ipfs_client(uri: &str) -> IpfsClient {
    // Parse the IPFS URL from the `--ipfs` command line argument
    let ipfs_address = if uri.starts_with("http://") || uri.starts_with("https://") {
        uri.to_string()
    } else {
        format!("http://{}", uri)
    };

    tracing::info!(ipfs_address, "Connect to IPFS node");

    //TODO: Test IPFS client

    match IpfsClient::new(&ipfs_address) {
        Ok(ipfs_client) => ipfs_client,
        Err(e) => {
            tracing::error!(
                msg = tracing::field::debug(&e),
                "Failed to create IPFS client",
            );
            panic!("Could not connect to IPFS");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_retry::strategy::{jitter, ExponentialBackoff};
    use tokio_retry::Retry;

    // fn test_client() -> IpfsClient {
    //     IpfsClient::new("https://ipfs.network.thegraph.com")
    // }

    #[tokio::test]
    async fn fetch_random_subgraph_yaml() {
        let ipfs_hash = "Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX";
        // https://ipfs.network.thegraph.com/api/v0/cat?arg=Qmc1mmagMJqopw2zb1iUTPRMhahMvEAKpQGS3KvuL9cpaX
        let client = create_ipfs_client("https://ipfs.network.thegraph.com");

        let retry_strategy = ExponentialBackoff::from_millis(10)
            .map(jitter) // add jitter to delays
            .take(5); // limit to 5 retries
        let timeout = Duration::from_secs(30);

        let file_bytes = Retry::spawn(retry_strategy, || client.cat_all(ipfs_hash, timeout))
            .await
            .unwrap();
        assert_ne!(file_bytes.len(), 0);
    }
}
