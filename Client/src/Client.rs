use std::fmt::format;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;
use std::env;




fn device_name() -> String {
    env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}




fn main() -> std::io::Result<()> {

     println!("OS détecté: {}", env::consts::OS);

    let dur = 5;
    
    let my_uuid = Uuid::new_v4();
    println!("UUID genere : {}", my_uuid);
    
    loop {       
        let mut stream = match TcpStream::connect("172.27.212.88:7878") {
            Ok(s) => s, 
            Err(e) => {
                println!("Impossible de se connecter : {}.", e);
                println!("Client Retry in {}s", dur);
                sleep(Duration::from_secs(dur));
                continue;
            }
        };


        let mut buffer = [0; 512];
        loop {
            println!("Connected successfull");
            match stream.read(&mut buffer) {
                Ok(size) => {
                    if size > 0 {
                        let message = String::from_utf8_lossy(&buffer[..size]);
                        let message = message.trim();
                        
                        if message == "whoareyou" {
                            
                            println!("Whoiam");
                            let uuidBytes = format!("{}\n", my_uuid).into_bytes();
                            let _ = stream.write_all(&uuidBytes);
    
    
                        } else if message == "getOs" {
                            let os  = env::consts::OS;
                            let osBytes = format!("{}\n", os).into_bytes();
                            let _ = stream.write_all(&osBytes);

                        } else if message == "getName" {
                            println!("getName If");
                            let name = device_name();
                            let nameFormated = format!("{}\n", name).into_bytes();
                            let _ = stream.write_all(&nameFormated);
                        } 

                        else  {
                            
                            println!("Message recu du server : {}", message);
                        }
    
                    }
                }
                Err(e) => {
                    println!("Erreur en lecture: {}", e);
                    break;
                }
            }
    
        }
        
        
        println!("Client Retry in {}s", dur);
        sleep(Duration::from_secs(dur));
    }




}
