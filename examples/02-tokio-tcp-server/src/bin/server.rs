use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::{BufReader, BufWriter};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};

async fn handle_client(socket: TcpStream) {
    let peer = match socket.peer_addr() {
        Ok(peer) => peer,
        Err(error) => {
            eprintln!("failed to get peer address: {error}");
            return;
        }
    };
    println!("client connected: {peer:?}");
    let (read_half, write_half) = socket.into_split();
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);
    let mut buffer = [0; 1024];
    let mut pending = Vec::new();
    loop {
        let result = timeout(Duration::from_secs(10), reader.read(&mut buffer)).await;

        match result {
            Ok(Ok(n)) if n == 0 => {
                println!("client disconnected: {peer:?}");
                break;
            }
            Ok(Ok(n)) if n > 0 => {
                pending.extend_from_slice(&buffer[..n]);

                while let Some(pos) = pending.iter().position(|&b| b == b'\n') {
                    let message: Vec<u8> = pending.drain(..=pos).collect();

                    let message = String::from_utf8_lossy(&message);
                    let message = message.trim_end();

                    println!("received from {peer:?}: {message}");

                    let response = format!("server received: {message}\n");

                    if let Err(error) = writer.write_all(response.as_bytes()).await {
                        eprintln!("failed to write to {peer:?}: {error}");
                        break;
                    }

                    if let Err(error) = writer.flush().await {
                        eprintln!("failed to flush to {peer:?}: {error}");
                        break;
                    }
                }
            }
            Ok(Ok(_)) => {
                eprintln!("unexpected read result: {result:?}");
                break;
            }
            Ok(Err(error)) => {
                eprintln!("read error from {peer:?}: {error}");
                break;
            }
            Err(_) => {
                eprintln!("client timeout: {peer:?}");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("server is running on port 8080");
    loop {
        let (socket, _) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("failed to accept connection: {error}");
                continue;
            }
        };
        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}
