mod bprotocol;
pub mod dht;
mod persistence;
mod routing_table;
mod utils;

pub use crate::util::dht::dht::{Dht, DhtConfig};
// pub use librqbit_core::id20::Id20;
pub use persistence::{PersistentDht, PersistentDhtConfig};

pub static DHT_BOOTSTRAP: &[&str] = &["dht.transmissionbt.com:6881", "dht.libtorrent.org:25401"];
