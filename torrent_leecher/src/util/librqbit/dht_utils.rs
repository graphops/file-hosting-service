use std::{collections::HashSet, net::SocketAddr};

use crate::util::buffers::ByteString;
use crate::util::librqbit_core::torrent_metainfo::TorrentMetaV1Info;
use anyhow::Context;
use futures::{stream::FuturesUnordered, Stream, StreamExt};
use log::debug;

// use crate::lib::{
//     peer_connection::PeerConnectionOptions, spawn_utils::BlockingSpawner,
// };
use crate::util::librqbit_core::id20::Id20;

use super::{
    peer_connection::PeerConnectionOptions, peer_info_reader, spawn_utils::BlockingSpawner,
};

#[derive(Debug)]
pub enum ReadMetainfoResult<Rx> {
    Found {
        info: TorrentMetaV1Info<ByteString>,
        rx: Rx,
        seen: HashSet<SocketAddr>,
    },
    ChannelClosed {
        seen: HashSet<SocketAddr>,
    },
}

pub async fn read_metainfo_from_peer_receiver<A: Stream<Item = SocketAddr> + Unpin>(
    peer_id: Id20,
    info_hash: Id20,
    mut addrs: A,
    peer_connection_options: Option<PeerConnectionOptions>,
) -> ReadMetainfoResult<A> {
    let mut seen = HashSet::<SocketAddr>::new();
    let first_addr = match addrs.next().await {
        Some(addr) => addr,
        None => return ReadMetainfoResult::ChannelClosed { seen },
    };
    seen.insert(first_addr);

    let semaphore = tokio::sync::Semaphore::new(128);

    let read_info_guarded = |addr| {
        let semaphore = &semaphore;
        async move {
            let token = semaphore.acquire().await?;
            let ret = peer_info_reader::read_metainfo_from_peer(
                addr,
                peer_id,
                info_hash,
                peer_connection_options,
                BlockingSpawner::new(true),
            )
            .await
            .with_context(|| format!("error reading metainfo from {addr}"));
            drop(token);
            ret
        }
    };

    let mut unordered = FuturesUnordered::new();
    unordered.push(read_info_guarded(first_addr));

    loop {
        tokio::select! {
            next_addr = addrs.next() => {
                match next_addr {
                    Some(addr) => {
                        if seen.insert(addr) {
                            unordered.push(read_info_guarded(addr));
                        }
                    },
                    None => return ReadMetainfoResult::ChannelClosed { seen },
                }
            },
            done = unordered.next(), if !unordered.is_empty() => {
                match done {
                    Some(Ok(info)) => return ReadMetainfoResult::Found { info, seen, rx: addrs },
                    Some(Err(e)) => {
                        debug!("{:#}", e);
                    },
                    None => unreachable!()
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    // use crate::lib::librqbit_core::dht::{Dht, Id20};

    use crate::util::{dht::Dht, librqbit_core::peer_id::generate_peer_id};

    use super::*;
    use std::{str::FromStr, sync::Once};

    static LOG_INIT: Once = Once::new();

    fn init_logging() {
        #[allow(unused_must_use)]
        LOG_INIT.call_once(|| {
            pretty_env_logger::try_init();
        })
    }

    #[tokio::test]
    async fn read_metainfo_from_dht() {
        init_logging();

        let info_hash = Id20::from_str("cf3ea75e2ebbd30e0da6e6e215e2226bf35f2e33").unwrap();
        let dht = Dht::new().await.unwrap();
        let peer_rx = dht.get_peers(info_hash).await.unwrap();
        let peer_id = generate_peer_id();
        match read_metainfo_from_peer_receiver(peer_id, info_hash, peer_rx, None).await {
            ReadMetainfoResult::Found { info, .. } => dbg!(info),
            ReadMetainfoResult::ChannelClosed { .. } => todo!("should not have happened"),
        };
    }
}
