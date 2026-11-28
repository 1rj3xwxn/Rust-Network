use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;

const CHANNEL_CAPACITY: usize = 100;

type ChatTx = broadcast::Sender<String>;

#[tokio::main]
async fn main() {
    // One shared broadcast channel for all connected clients.
    let (tx, _) = broadcast::channel::<String>(CHANNEL_CAPACITY);

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
        .with_state(tx);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("failed to bind TCP listener");

    println!("WebSocket chat listening on http://127.0.0.1:8080");

    axum::serve(listener, app)
        .await
        .expect("server failed");
}

async fn index() -> &'static str {
    "WebSocket chat server is running. Connect a WebSocket client to /ws."
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(tx): State<ChatTx>,
) -> impl IntoResponse {
    // The HTTP upgrade is completed before handle_socket starts.
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: ChatTx) {
    // Every connection gets its own receiver for the shared channel.
    let mut rx = tx.subscribe();

    // Split the socket so reading and writing can progress independently.
    let (mut sender, mut receiver) = socket.split();

    println!("WebSocket client connected");

    loop {
        tokio::select! {
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        println!("Received: {text}");

                        // Publish the message. All active subscribers,
                        // including the sender, can receive this message.
                        if tx.send(text.to_string()).is_err() {
                            eprintln!("No active broadcast receivers");
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        println!("WebSocket client disconnected");
                        break;
                    }
                    Some(Err(error)) => {
                        eprintln!("WebSocket receive error: {error}");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Binary and control messages are not used in this
                        // small text-chat example.
                    }
                }
            }

            outgoing = rx.recv() => {
                match outgoing {
                    Ok(text) => {
                        if sender.send(Message::Text(text.into())).await.is_err() {
                            println!("WebSocket client send failed");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        // The receiver was too slow and older messages were
                        // overwritten in the bounded broadcast buffer.
                        eprintln!("Client lagged; skipped {skipped} messages");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        println!("Broadcast channel closed");
                        break;
                    }
                }
            }
        }
    }

    println!("WebSocket connection task finished");
}
