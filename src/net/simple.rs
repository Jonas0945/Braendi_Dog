/*use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
use serde_json::Value as RawMessage;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex as TokioMutex;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::net::transport::{NetworkEvent, NetworkTransport};

#[derive(Clone)]
pub struct SimpleTcpTransport {
    peers: Arc<TokioMutex<HashMap<usize, UnboundedSender<Bytes>>>>,
    event_senders: Arc<Mutex<Vec<UnboundedSender<NetworkEvent>>>>,
    next_id: Arc<AtomicUsize>,
    is_shutdown: Arc<AtomicUsize>,
}

impl SimpleTcpTransport {
    pub fn new() -> Arc<Self> {
        Arc::new(SimpleTcpTransport {
            peers: Arc::new(TokioMutex::new(HashMap::new())),
            event_senders: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(AtomicUsize::new(1)),
            is_shutdown: Arc::new(AtomicUsize::new(0)),
        })
    }

    fn next_peer_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn push_event(&self, ev: NetworkEvent) {
        let senders = {
            let guard = self.event_senders.lock().unwrap();
            guard.clone()
        };
        for s in senders {
            let _ = s.unbounded_send(ev.clone());
        }
    }
}

pub async fn start_server(addr: &str) -> Result<Arc<SimpleTcpTransport>> {
    let transport = SimpleTcpTransport::new();
    let listener = TcpListener::bind(addr).await?;
    let t = transport.clone();
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    let id = t.next_peer_id();
                    t.push_event(NetworkEvent::Connected(id));
                    handle_connection(socket, id, t.clone());
                }
                Err(e) => {
                    t.push_event(NetworkEvent::Error(format!("accept error: {}", e)));
                    break;
                }
            }
        }
    });
    Ok(transport)
}

pub async fn start_client(addr: &str) -> Result<Arc<SimpleTcpTransport>> {
    let transport = SimpleTcpTransport::new();
    let sock = TcpStream::connect(addr).await?;
    let id = transport.next_peer_id();
    transport.push_event(NetworkEvent::Connected(id));
    handle_connection(sock, id, transport.clone());
    Ok(transport)
}

fn handle_connection(socket: TcpStream, id: usize, transport: Arc<SimpleTcpTransport>) {
    tokio::spawn(async move {
        let (r, w) = socket.into_split();
        let mut framed_read = FramedRead::new(r, LengthDelimitedCodec::new());
        let mut framed_write = FramedWrite::new(w, LengthDelimitedCodec::new());

        // writer channel
        let (tx, mut rx): (UnboundedSender<Bytes>, UnboundedReceiver<Bytes>) = unbounded();
        {
            let mut peers = transport.peers.lock().await;
            peers.insert(id, tx);
        }

        // Read loop
        let t1 = transport.clone();
        let read_task = tokio::spawn(async move {
            while let Some(Ok(bytes)) = framed_read.next().await {
                match serde_json::from_slice::<RawMessage>(&bytes) {
                    Ok(msg) => t1.push_event(NetworkEvent::Message(id, msg)),
                    Err(e) => t1.push_event(NetworkEvent::Error(format!("json parse: {}", e))),
                }
            }
            t1.push_event(NetworkEvent::Disconnected(id));
            let mut peers = t1.peers.lock().await;
            peers.remove(&id);
        });

        // Write loop
        let write_task = tokio::spawn(async move {
            while let Some(bytes) = rx.next().await {
                if let Err(e) = framed_write.send(bytes).await {
                    // typically connection broken
                    eprintln!("write error {}", e);
                    break;
                }
            }
        });

        // detach tasks
        let _ = tokio::join!(read_task, write_task);
    });
}

#[async_trait]
impl NetworkTransport for Arc<SimpleTcpTransport> {
    async fn send(&self, peer: usize, msg: &RawMessage) -> Result<()> {
        let serialized = serde_json::to_vec(msg)?;
        let bytes = Bytes::from(serialized);
        let peers = self.peers.lock().await;
        if let Some(tx) = peers.get(&peer) {
            tx.unbounded_send(bytes).map_err(|e| anyhow::anyhow!("send error: {}", e))?;
        } else {
            return Err(anyhow::anyhow!("peer not found"));
        }
        Ok(())
    }

    async fn broadcast(&self, msg: &RawMessage) -> Result<()> {
        let serialized = serde_json::to_vec(msg)?;
        let bytes = Bytes::from(serialized);
        let peers = self.peers.lock().await;
        for (_id, tx) in peers.iter() {
            let _ = tx.unbounded_send(bytes.clone());
        }
        Ok(())
    }

    fn subscribe(&self) -> UnboundedReceiver<NetworkEvent> {
        let (tx, rx) = unbounded();
        let mut guard = self.event_senders.lock().unwrap();
        guard.push(tx);
        rx
    }

    async fn shutdown(&self) -> Result<()> {
        self.is_shutdown.store(1, Ordering::Relaxed);
        // drop peers
        let mut peers = self.peers.lock().await;
        peers.clear();
        Ok(())
    }
}
*/