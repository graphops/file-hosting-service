use bytes::Bytes;
use futures::StreamExt;

use object_store::{parse_url_opts, path::Path, ObjectStore};
use reqwest::Url;
use tokio::io::AsyncWriteExt;

use object_store::{ObjectMeta, PutResult};

use std::sync::Arc;

use crate::manifest::Error;

pub fn s3_store(
    endpoint: &str,
    region: &str,
    s3_endpoint: &str,
    bucket: &str,
    access_key_id: &str,
    secret_key: &str,
) -> Result<(Arc<Box<dyn ObjectStore>>, Path), Error> {
    let url = Url::parse(endpoint).unwrap();
    parse_url_opts(
        &url,
        vec![
            ("region", region),
            ("endpoint", s3_endpoint),
            ("bucket", bucket),
            ("aws_access_key_id", access_key_id),
            ("aws_secret_access_key", secret_key),
        ],
    )
    .map_err(Error::ObjectStoreError)
    .map(|(s, p)| (Arc::new(s), p))
}

pub async fn list(store: Arc<Box<dyn ObjectStore>>) -> Result<Vec<ObjectMeta>, Error> {
    let mut object_metas = vec![];
    let mut list_stream = store.list(None);

    while let Ok(Some(meta)) = list_stream.next().await.transpose() {
        tracing::debug!("Name: {}, size: {}", meta.location, meta.size);
        object_metas.push(meta);
    }
    Ok(object_metas)
}

pub async fn write(
    store: Arc<Box<dyn ObjectStore>>,
    bytes: Bytes,
    loc: &str,
) -> Result<PutResult, Error> {
    let location = Path::from(loc);
    let put_result = store
        .put(&location, bytes)
        .await
        .map_err(Error::ObjectStoreError)?;

    Ok(put_result)
}

pub async fn delete(store: Arc<Box<dyn ObjectStore>>, loc: &str) -> Result<(), Error> {
    let location = Path::from(loc);
    store
        .delete(&location)
        .await
        .map_err(Error::ObjectStoreError)?;
    Ok(())
}

pub async fn multipart_write(
    store: Arc<Box<dyn ObjectStore>>,
    bytes: Bytes,
    loc: &str,
) -> Result<(), Error> {
    let location = Path::from(loc);
    let (_id, mut writer) = store.put_multipart(&location).await.unwrap();

    writer.write_all(&bytes).await.unwrap();
    writer.flush().await.unwrap();
    writer.shutdown().await.unwrap();

    Ok(())
}
