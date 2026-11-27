use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(mut stream: TcpStream, client_id: u32, count_clone: Arc<Mutex<u32>>) {
    let peer = stream.peer_addr().ok();
    println!("client connected: {peer:?}");

    let reader_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("failed to clone stream: {error}");
            let mut count_value = count_clone.lock().unwrap();
            *count_value -= 1;
            println!("client count: {}", *count_value);
            return;
        }
    };

    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("client disconnected: {peer:?} (client {client_id})");
                break;
            }
            Ok(_) => {
                let message = line.trim_end();
                println!("received from {peer:?}: {message} (client {client_id})");

                if let Err(error) =
                    writeln!(stream, "server received: {message} (client {client_id})")
                {
                    eprintln!("failed to write to {peer:?}: {error} (client {client_id})");
                    break;
                }

                if let Err(error) = stream.flush() {
                    eprintln!("failed to flush to {peer:?}: {error} (client {client_id})");
                    break;
                }
            }
            Err(error) => {
                eprintln!("read error from {peer:?}: {error} (client {client_id})");
                break;
            }
        }
    }
    let mut count_value = count_clone.lock().unwrap();
    *count_value -= 1;
    println!("client count: {}", *count_value);
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("TCP server listening on 127.0.0.1:7878");
    let mut client_id = 0u32;
    let count = Arc::new(Mutex::new(0));

    for incoming in listener.incoming() {
        let count_clone = Arc::clone(&count);
        match incoming {
            Ok(stream) => {
                client_id += 1;
                {
                    let mut count_value = count_clone.lock().unwrap();
                    *count_value += 1;
                    println!("client count: {}", *count_value);
                }
                thread::spawn(move || handle_client(stream, client_id, count_clone));
            }
            Err(error) => {
                eprintln!("failed to accept connection: {error}");
            }
        }
    }

    Ok(())
}
