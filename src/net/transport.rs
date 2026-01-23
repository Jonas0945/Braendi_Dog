/*use async_trait::async_trait;
use futures::channel::mpsc::UnboundedReceiver;
use serde_json::Value as RawMessage;

pub type RawMessage = RawMessage;
//Interface, erhält nachrichten wie Connection successful und Fehler 
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    Connected(usize),
    Disconnected(usize),
    Message(usize, RawMessage),
    Error(String),
}
//Was das Netzwerk können muss (send: an einen naderen Spieler), broadcast: an alle und subscribe, das tun die clients um Nachrichten zu empfangen
#[async_trait]
pub trait NetworkTransport: Send + Sync + 'static {
    async fn send(&self, peer: usize, msg: &RawMessage) -> anyhow::Result<()>;
    async fn broadcast(&self, msg: &RawMessage) -> anyhow::Result<()>;
    fn subscribe(&self) -> UnboundedReceiver<NetworkEvent>;
    async fn shutdown(&self) -> anyhow::Result<()>;
}*/
