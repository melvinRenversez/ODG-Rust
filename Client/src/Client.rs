use std::net::TcpStream;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    println!("Connected to the server");

    stream.write_all(b"hello server");
    println!("send message");

    Ok(())
}
