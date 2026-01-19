# Braendi_Dog 

## Networking (for GUI / multiplayer)

Quick example for GUI developers showing how to start a server, subscribe to events and send messages (JSON):

```rust
use serde_json::json;
use braendi_dog::net::{start_server, start_client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// server side
	let server = start_server("127.0.0.1:4000").await?;
	let mut rx = server.subscribe();

	// spawn a task to handle incoming network events
	tokio::spawn(async move {
		while let Some(ev) = rx.next().await {
			println!("network event: {:?}", ev);
		}
	});

	// broadcast a JSON message to connected peers
	server.broadcast(&json!({"type":"hello","ts":123})).await?;

	Ok(())
}
```

GUI devs should treat messages as JSON blobs (`serde_json::Value`). For structured game messages, agree on a JSON schema and (de)serialize into your game types.
