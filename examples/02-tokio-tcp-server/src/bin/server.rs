use tokio::io::{BufReader, BufWriter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

async fn handle_client(socket: TcpStream) {
    let peer = socket.peer_addr().unwrap();
    let (read_half, write_half) = socket.into_split();
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);
    let mut line = String::new();
    loop {
        line.clear();

        match reader.read_line( &mut line).await {
            Ok(0) => {
                println!("client disconnected: {peer:?}");
                break;
            }
            Ok(_) => {
                let message = line.trim_end();
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
            Err(error) => {
                eprintln!("read error from {peer:?}: {error}");
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
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_client(socket).await;
        });
    }
}
