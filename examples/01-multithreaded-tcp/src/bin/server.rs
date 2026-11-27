use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let peer = stream.peer_addr().ok();
    println!("client connected: {peer:?}");

    let reader_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("failed to clone stream: {error}");
            return;
        }
    };

    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("client disconnected: {peer:?}");
                break;
            }
            Ok(_) => {
                let message = line.trim_end();
                println!("received from {peer:?}: {message}");

                if let Err(error) = writeln!(stream, "server received: {message}") {
                    eprintln!("failed to write to {peer:?}: {error}");
                    break;
                }

                if let Err(error) = stream.flush() {
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

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("TCP server listening on 127.0.0.1:7878");

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(error) => {
                eprintln!("failed to accept connection: {error}");
            }
        }
    }

    Ok(())
}
