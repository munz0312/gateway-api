use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tokio::time::Duration; // Removed `timeout` import as it's no longer needed.

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:3000/ws";
    println!("Connecting to {}", url);
    println!("Press Ctrl+C to exit.");

    // Give server a second to be ready if we just started it
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to WebSocket. Listening for messages...");

    let (_write, mut read) = ws_stream.split();

    // Spawn a task to send HTTP requests to trigger metrics
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("\n--- Sending initial HTTP request to trigger metrics... ---");
        let client = reqwest::Client::new();
        // Hit a few times to get some logs and ensure metrics are updated
        for i in 0..3 {
            let res = client.get("http://127.0.0.1:3000/health").send().await;
            match res {
                Ok(r) => println!("HTTP Request {} status: {}", i + 1, r.status()),
                Err(e) => println!("HTTP Request {} failed: {}", i + 1, e),
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        println!("--- Initial HTTP requests sent. ---");
    });

    // Continuously listen for messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                if msg.is_text() {
                    println!("\nReceived WS message: {}", msg.to_text().unwrap_or("[unreadable text]"));
                } else if msg.is_ping() {
                    println!("\nReceived WS Ping");
                } else if msg.is_pong() {
                    println!("\nReceived WS Pong");
                } else if msg.is_close() {
                    println!("\nWebSocket connection closed by server.");
                    break;
                }
            }
            Err(e) => {
                eprintln!("\nWebSocket error: {:?}", e);
                break;
            }
        }
    }
    println!("WebSocket monitor stopped.");
}