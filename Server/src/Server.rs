use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

struct Client {
    uuid: Option<Uuid>,
    os: Option<String>,
    name: Option<String>,
    stream: Arc<Mutex<TcpStream>>,
}

#[derive(Serialize)]
struct ClientInfo {
    uuid: Option<Uuid>,
    os: Option<String>,
    name: Option<String>,
}

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<Client>>>) {
    println!("New Message by {:?}", stream.peer_addr());

    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    println!("Client desconnected");

                    clients.lock().unwrap().retain(|c| {
                        c.stream.lock().unwrap().peer_addr().unwrap() != stream.peer_addr().unwrap()
                    });
                    break;
                } else {
                    let message = String::from_utf8_lossy(&buffer[..size]);
                    println!("Recieves: {}", message);
                }
            }
            Err(e) => {
                println!("Error reading: {}", e);
                clients.lock().unwrap().retain(|c| {
                    c.stream.lock().unwrap().peer_addr().unwrap() != stream.peer_addr().unwrap()
                });
                println!("Client restant : {}", clients.lock().unwrap().len());
                break;
            }
        }
    }
}

fn getClientData(mut stream: &TcpStream, clients: &Arc<Mutex<Vec<Client>>>) {
    println!("getClientData");

    stream.try_clone().unwrap().write_all(b"whoareyou");
    let mut buffer = [0; 64];
    let size = stream.read(&mut buffer).unwrap();
    let uuidStr = String::from_utf8_lossy(&buffer[..size]);
    let clientUuid = Uuid::parse_str(uuidStr.trim()).ok();

    stream.try_clone().unwrap().write_all(b"getOs");
    let mut buffer = [0; 255];
    let size = stream.read(&mut buffer).unwrap();
    let osStr = String::from_utf8_lossy(&buffer[..size]).trim().to_string();

    stream.try_clone().unwrap().write_all(b"getName");
    let mut buffer = [0; 255];
    let size = stream.read(&mut buffer).unwrap();
    let nameStr = String::from_utf8_lossy(&buffer[..size]).trim().to_string();



    println!("Client uuid : {} , client os : {}, name : {}", uuidStr, osStr, nameStr);

    clients.lock().unwrap().push(Client {
        uuid: clientUuid,
        os: Some(osStr),
        name: Some(nameStr),
        stream: Arc::new(Mutex::new(stream.try_clone().unwrap())),
    });

    stream.write_all(b"test data cleitn");
}

#[get("/")]
async fn homePage() -> impl Responder {
    let html = fs::read_to_string("src/index.html")
        .unwrap_or_else(|_| "Error: fichier introuvable".to_string());

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[get("/msg")]
async fn msg(
    query: web::Query<std::collections::HashMap<String, String>>,
    clients: web::Data<Arc<Mutex<Vec<Client>>>>,
) -> impl Responder {
    let message = query
        .get("message")
        .map(|s| s.to_string())
        .unwrap_or_else(|| "no".to_string());
    println!("Received message : {}", message);

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
async fn getDevices(clients: web::Data<Arc<Mutex<Vec<Client>>>>) -> impl Responder {
    println!("Get Devices");

    let clientList = getClientForWeb(&clients);

    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&clientList).unwrap())
}

fn clientToInfo(Client: &Client) -> ClientInfo {
    ClientInfo {
        uuid: Client.uuid,
        os: Client.os.clone(),
        name: Client.name.clone(),
    }
}

fn getClientForWeb(clients: &Arc<Mutex<Vec<Client>>>) -> Vec<ClientInfo> {
    let ClientLock = clients.lock().unwrap();

    ClientLock.iter().map(|c| clientToInfo(c)).collect()
}

fn start_tcp_server(clients: Arc<Mutex<Vec<Client>>>) {
    thread::spawn(move || {
        let listener = match TcpListener::bind("0.0.0.0:7878") {
            Ok(listener) => listener,
            Err(e) => {
                println!("Failled to binf TCP {}", e);
                return;
            }
        };
        println!("Server open on 0.0.0.0:7878...");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("New Connexion");

                    getClientData(&stream, &clients);

                    let clients_clone = clients.clone();
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
            .service(getDevices)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

// envoie de message entre server et client avec reponse
