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

#[cfg(test)]

mod tests {

    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use bytes::Buf;

    use crate::test_util::{create_random_temp_file, random_bytes, CHUNK_SIZE};

    use super::*;

    #[tokio::test]

    // #[ignore]

    async fn test_list_file() {
        let region = "";

        let bucket = "";

        let access_key_id = "";

        let secret_key = "";

        let endpoint = "s3://";

        let s3_endpoint = "";

        let (store, _) = s3_store(
            endpoint,
            region,
            s3_endpoint,
            bucket,
            access_key_id,
            secret_key,
        )
        .unwrap();

        println!("test list file: {store:?}");

        let res = list(store).await;

        println!("{:#?}", res);

        assert!(res.is_ok());
    }

    #[tokio::test]

    async fn test_write_file() {
        let file_size = CHUNK_SIZE * 25;
        let bytes = random_bytes(file_size.try_into().unwrap());
        let region = "";

        let bucket = "";

        let access_key_id = "";

        let secret_key = "";

        let endpoint = "s3://";

        let s3_endpoint = "";

        let (store, _) = s3_store(
            endpoint,
            region,
            s3_endpoint,
            bucket,
            access_key_id,
            secret_key,
        )
        .unwrap();

        let location = "test_upload_file.txt";
        let res = write(store.clone(), bytes.into(), location).await;
        assert!(res.is_ok());
        let res = delete(store.clone(), location).await;
        assert!(res.is_ok());
        let bytes = random_bytes(file_size.try_into().unwrap());
        let res = multipart_write(store.clone(), bytes.into(), location).await;
        assert!(res.is_ok());
        let res = delete(store.clone(), location).await;
        assert!(res.is_ok());
        let res = list(store).await;
        assert!(res.is_ok());
    }
}
