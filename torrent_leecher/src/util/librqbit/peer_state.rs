use std::{collections::HashSet, sync::Arc};

use crate::util::librqbit_core::id20::Id20;
use crate::util::librqbit_core::lengths::{ChunkInfo, ValidPieceIndex};
use tokio::sync::{Notify, Semaphore};

use crate::util::librqbit::type_aliases::BF;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct InflightRequest {
    pub piece: ValidPieceIndex,
    pub chunk: u32,
}

impl From<&ChunkInfo> for InflightRequest {
    fn from(c: &ChunkInfo) -> Self {
        Self {
            piece: c.piece_index,
            chunk: c.chunk_index,
        }
    }
}

#[derive(Debug)]
pub enum PeerState {
    Queued,
    Connecting,
    Live(LivePeerState),
}

#[derive(Debug)]
pub struct LivePeerState {
    pub peer_id: Id20,
    pub i_am_choked: bool,
    pub peer_interested: bool,
    pub requests_sem: Arc<Semaphore>,
    pub have_notify: Arc<Notify>,
    pub bitfield: Option<BF>,
    pub inflight_requests: HashSet<InflightRequest>,
}

impl LivePeerState {
    pub fn new(peer_id: Id20) -> Self {
        LivePeerState {
            peer_id,
            i_am_choked: true,
            peer_interested: false,
            bitfield: None,
            have_notify: Arc::new(Notify::new()),
            requests_sem: Arc::new(Semaphore::new(0)),
            inflight_requests: Default::default(),
        }
    }
}
