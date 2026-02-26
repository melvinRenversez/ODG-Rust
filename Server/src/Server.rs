use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::thread;
use std::fs;
use uuid::Uuid;

struct Client{
    uuid: Option<Uuid>,
    stream: Arc<Mutex<TcpStream>>,
}

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<Client>>>) {
    println!("New Message by {:?}", stream.peer_addr());

    let mut buffer = [0; 512];
    loop{
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    println!("Client desconnected");
                    
                    clients.lock().unwrap().retain(|c| c.stream.lock().unwrap().peer_addr().unwrap() != stream.peer_addr().unwrap());
                    break;

                }else {
                    let message = String::from_utf8_lossy(&buffer[..size]);
                    println!("Recieves: {}", message);
                }
            }
            Err(e) => {
                println!("Error reading: {}", e);
                clients.lock().unwrap().retain(|c| c.stream.lock().unwrap().peer_addr().unwrap() != stream.peer_addr().unwrap());
                println!("Client restant : {}", clients.lock().unwrap().len());
                break;
            } 
        }
    }
}

#[get("/")]
async fn homePage() -> impl Responder  {

    let html = fs::read_to_string("src/index.html")
        .unwrap_or_else(|_| "Error: fichier introuvable".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)

}

#[get("/msg")]
async fn msg(query: web::Query<std::collections::HashMap<String, String>>, clients: web::Data<Arc<Mutex<Vec<Client>>>>) -> impl Responder {
    let message = query.get("message")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "no".to_string());
    println!( "Received message : {}", message);

    let clients_lock = clients.lock().unwrap();
    for mut client in clients_lock.iter() {
        let mut stream = client.stream.lock().unwrap();
        let _ = stream.write_all(message.as_bytes());
    }

    HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish()
}

#[get("/getDevices")]
async fn getDevices() -> impl Responder {
    return "getDevices"
}


fn start_tcp_server(clients: Arc<Mutex<Vec<Client>>>) {

        thread::spawn(move || {
        let listener = match TcpListener::bind("0.0.0.0:7878")  {
            Ok(listener) => listener,
            Err(e) => {
                println!("Failled to binf TCP {}", e);
                return;
            }
        };
        println!("Server open on 0.0.0.0:7878...");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New Connexion");
                    clients.lock().unwrap().push(Client {
                        uuid: None,
                        stream: Arc::new(Mutex::new(stream.try_clone().unwrap())),
                    });
                    let clients_clone  = clients.clone();
                    thread::spawn(move || {
                        handle_client(stream, clients_clone);
                    });
                }
                Err(e) => println!("Connexion failed {}", e),
            }
        }
    });
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let clients = Arc::new(Mutex::new(Vec::new()));

    let tcp_clients = clients.clone();
    thread::spawn(move || {
        start_tcp_server(tcp_clients);
    });
    
    HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(clients.clone()))
                .service(homePage)
                .service(msg)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

// envoie de message entre server et client avec reponse