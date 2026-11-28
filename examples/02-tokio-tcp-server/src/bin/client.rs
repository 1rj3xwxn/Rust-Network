use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080")?;
    println!("connected to server; type a message and press Enter");
    println!("type /quit to exit");

    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);
    let stdin = io::stdin();
    let mut input = String::new();
    let mut response = String::new();

    loop {
        input.clear();
        print!("> ");
        io::stdout().flush()?;

        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        if input.trim() == "/quit" {
            break;
        }
        input = "12324\n3456\n7890\n".to_string();
        stream.write_all(input.as_bytes())?;
        stream.flush()?;

        response.clear();
        if reader.read_line(&mut response)? == 0 {
            println!("server closed the connection");
            break;
        }

        print!("{response}");
    }

    println!("client exiting");
    Ok(())
}
