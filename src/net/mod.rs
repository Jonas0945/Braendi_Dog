/*pub mod transport;
pub mod simple;

pub use transport::{NetworkEvent, RawMessage, NetworkTransport};
pub use simple::{SimpleTcpTransport, start_client, start_server};

// Short doc: GUI devs can call `start_server` or `start_client` and then
// `subscribe()` to receive `NetworkEvent`s; use `send`/`broadcast` to send JSON messages.
*/

pub mod client;
pub mod server;
