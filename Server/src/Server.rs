use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};


fn handle_client(mut stream: TcpStream) {
    println!("New Message by {:?}", stream.peer_addr());

    let mut buffer = [0; 512];
    match stream.read(&mut buffer) {
        Ok(size) => {
            if size == 0 {
                println!("Client desconnected");
            }else {
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("Recieves: {}", message);
            }
        }
        Err(e) => println!("Error reading: {}", e),
    }
}


fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server open on 127.0.0.1:7878...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New Connexion");
                handle_client(stream);
            }
            Err(e) => println!("Connexion failed {}", e),
        }
    }

    Ok(())
}
