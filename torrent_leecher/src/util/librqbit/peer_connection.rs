use std::{net::SocketAddr, time::Duration};

use crate::util::buffers::{ByteBuf, ByteString};
use crate::util::clone_to_owned::CloneToOwned;
use crate::util::librqbit_core::{id20::Id20, lengths::ChunkInfo, peer_id::try_decode_peer_id};
use crate::util::peer_binary_protocol::{
    extended::{handshake::ExtendedHandshake, ExtendedMessage},
    serialize_piece_preamble, Handshake, Message, MessageBorrowed, MessageDeserializeError,
    MessageOwned, PIECE_MESSAGE_DEFAULT_LEN,
};
use anyhow::Context;
use log::{debug, trace};
use tokio::time::timeout;

use super::spawn_utils::BlockingSpawner;

pub trait PeerConnectionHandler {
    fn get_have_bytes(&self) -> u64;
    fn serialize_bitfield_message_to_buf(&self, buf: &mut Vec<u8>) -> Option<usize>;
    fn on_handshake(&self, handshake: Handshake) -> anyhow::Result<()>;
    fn on_extended_handshake(
        &self,
        extended_handshake: &ExtendedHandshake<ByteBuf>,
    ) -> anyhow::Result<()>;
    fn on_received_message(&self, msg: Message<ByteBuf<'_>>) -> anyhow::Result<()>;
    fn on_uploaded_bytes(&self, bytes: u32);
    fn read_chunk(&self, chunk: &ChunkInfo, buf: &mut [u8]) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub enum WriterRequest {
    Message(MessageOwned),
    ReadChunkRequest(ChunkInfo),
}

#[derive(Default, Copy, Clone)]
pub struct PeerConnectionOptions {
    pub connect_timeout: Option<Duration>,
    pub read_write_timeout: Option<Duration>,
    pub keep_alive_interval: Option<Duration>,
}

pub struct PeerConnection<H> {
    handler: H,
    addr: SocketAddr,
    info_hash: Id20,
    peer_id: Id20,
    options: PeerConnectionOptions,
    spawner: BlockingSpawner,
}

async fn with_timeout<T, E>(
    timeout_value: Duration,
    fut: impl std::future::Future<Output = Result<T, E>>,
) -> anyhow::Result<T>
where
    E: Into<anyhow::Error>,
{
    timeout(timeout_value, fut)
        .await
        .with_context(|| format!("timeout at {timeout_value:?}"))?
        .map_err(|e| e.into())
}

macro_rules! read_one {
    ($conn:ident, $read_buf:ident, $read_so_far:ident, $rwtimeout:ident) => {{
        let (extended, size) = loop {
            match MessageBorrowed::deserialize(&$read_buf[..$read_so_far]) {
                Ok((msg, size)) => break (msg, size),
                Err(MessageDeserializeError::NotEnoughData(d, _)) => {
                    if $read_buf.len() < $read_so_far + d {
                        $read_buf.reserve(d);
                        $read_buf.resize($read_buf.capacity(), 0);
                    }

                    let size = with_timeout($rwtimeout, $conn.read(&mut $read_buf[$read_so_far..]))
                        .await
                        .context("error reading from peer")?;
                    if size == 0 {
                        anyhow::bail!("disconnected while reading, read so far: {}", $read_so_far)
                    }
                    $read_so_far += size;
                }
                Err(e) => return Err(e.into()),
            }
        };
        (extended, size)
    }};
}

impl<H: PeerConnectionHandler> PeerConnection<H> {
    pub fn new(
        addr: SocketAddr,
        info_hash: Id20,
        peer_id: Id20,
        handler: H,
        options: Option<PeerConnectionOptions>,
        spawner: BlockingSpawner,
    ) -> Self {
        PeerConnection {
            handler,
            addr,
            info_hash,
            peer_id,
            spawner,
            options: options.unwrap_or_default(),
        }
    }
    pub fn into_handler(self) -> H {
        self.handler
    }
    pub async fn manage_peer(
        &self,
        mut outgoing_chan: tokio::sync::mpsc::UnboundedReceiver<WriterRequest>,
    ) -> anyhow::Result<()> {
        use tokio::io::AsyncReadExt;
        use tokio::io::AsyncWriteExt;

        let rwtimeout = self
            .options
            .read_write_timeout
            .unwrap_or_else(|| Duration::from_secs(10));

        let connect_timeout = self
            .options
            .connect_timeout
            .unwrap_or_else(|| Duration::from_secs(10));

        let mut conn = with_timeout(connect_timeout, tokio::net::TcpStream::connect(self.addr))
            .await
            .context("error connecting")?;

        let mut write_buf = Vec::<u8>::with_capacity(PIECE_MESSAGE_DEFAULT_LEN);
        let handshake = Handshake::new(self.info_hash, self.peer_id);
        handshake.serialize(&mut write_buf);
        with_timeout(rwtimeout, conn.write_all(&write_buf))
            .await
            .context("error writing handshake")?;
        write_buf.clear();

        let mut read_buf = vec![0u8; PIECE_MESSAGE_DEFAULT_LEN * 2];
        let mut read_so_far = with_timeout(rwtimeout, conn.read(&mut read_buf))
            .await
            .context("error reading handshake")?;
        if read_so_far == 0 {
            anyhow::bail!("bad handshake");
        }
        let (h, size) = Handshake::deserialize(&read_buf[..read_so_far])
            .map_err(|e| anyhow::anyhow!("error deserializing handshake: {:?}", e))?;

        debug!(
            "connected peer {}: {:?}",
            self.addr,
            try_decode_peer_id(Id20(h.peer_id))
        );
        if h.info_hash != self.info_hash.0 {
            anyhow::bail!("info hash does not match");
        }

        let mut extended_handshake: Option<ExtendedHandshake<ByteString>> = None;
        let supports_extended = h.supports_extended();

        self.handler.on_handshake(h)?;
        if read_so_far > size {
            read_buf.copy_within(size..read_so_far, 0);
        }
        read_so_far -= size;

        if supports_extended {
            let my_extended =
                Message::Extended(ExtendedMessage::Handshake(ExtendedHandshake::new()));
            trace!(
                "sending extended handshake to {}: {:?}",
                self.addr,
                &my_extended
            );
            my_extended.serialize(&mut write_buf, None).unwrap();
            with_timeout(rwtimeout, conn.write_all(&write_buf))
                .await
                .context("error writing extended handshake")?;
            write_buf.clear();

            let (extended, size) = read_one!(conn, read_buf, read_so_far, rwtimeout);
            match extended {
                Message::Extended(ExtendedMessage::Handshake(h)) => {
                    trace!("received from {}: {:?}", self.addr, &h);
                    self.handler.on_extended_handshake(&h)?;
                    extended_handshake = Some(h.clone_to_owned())
                }
                other => anyhow::bail!("expected extended handshake, but got {:?}", other),
            };

            if read_so_far > size {
                read_buf.copy_within(size..read_so_far, 0);
            }
            read_so_far -= size;
        }

        let (mut read_half, mut write_half) = tokio::io::split(conn);

        let writer = async move {
            let keep_alive_interval = self
                .options
                .keep_alive_interval
                .unwrap_or_else(|| Duration::from_secs(120));

            if self.handler.get_have_bytes() > 0 {
                if let Some(len) = self
                    .handler
                    .serialize_bitfield_message_to_buf(&mut write_buf)
                {
                    with_timeout(rwtimeout, write_half.write_all(&write_buf[..len]))
                        .await
                        .context("error writing bitfield to peer")?;
                    debug!("sent bitfield to {}", self.addr);
                }
            }

            loop {
                let req = match timeout(keep_alive_interval, outgoing_chan.recv()).await {
                    Ok(Some(msg)) => msg,
                    Ok(None) => {
                        anyhow::bail!("closing writer, channel closed")
                    }
                    Err(_) => WriterRequest::Message(MessageOwned::KeepAlive),
                };

                let mut uploaded_add = None;

                let len = match &req {
                    WriterRequest::Message(msg) => {
                        msg.serialize(&mut write_buf, extended_handshake.as_ref())?
                    }
                    WriterRequest::ReadChunkRequest(chunk) => {
                        // this whole section is an optimization
                        write_buf.resize(PIECE_MESSAGE_DEFAULT_LEN, 0);
                        let preamble_len = serialize_piece_preamble(chunk, &mut write_buf);
                        let full_len = preamble_len + chunk.size as usize;
                        write_buf.resize(full_len, 0);
                        self.spawner
                            .spawn_block_in_place(|| {
                                self.handler
                                    .read_chunk(chunk, &mut write_buf[preamble_len..])
                            })
                            .with_context(|| format!("error reading chunk {chunk:?}"))?;

                        uploaded_add = Some(chunk.size);
                        full_len
                    }
                };

                debug!("sending to {}: {:?}, length={}", self.addr, &req, len);

                with_timeout(rwtimeout, write_half.write_all(&write_buf[..len]))
                    .await
                    .context("error writing the message to peer")?;
                write_buf.clear();

                if let Some(uploaded_add) = uploaded_add {
                    self.handler.on_uploaded_bytes(uploaded_add)
                }
            }

            // For type inference.
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        };

        let reader = async move {
            loop {
                let (message, size) = read_one!(read_half, read_buf, read_so_far, rwtimeout);
                trace!("received from {}: {:?}", self.addr, &message);

                self.handler
                    .on_received_message(message)
                    .context("error in handler.on_received_message()")?;

                if read_so_far > size {
                    read_buf.copy_within(size..read_so_far, 0);
                }
                read_so_far -= size;
            }

            // For type inference.
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        };

        let r = tokio::select! {
            r = reader => {r}
            r = writer => {r}
        };
        debug!("{}: either reader or writer are done, exiting", self.addr);
        r
    }
}
