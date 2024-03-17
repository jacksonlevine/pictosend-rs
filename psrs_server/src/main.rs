use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

#[derive(Clone, Debug)]
struct TextureData {
    data: [u8; WIDTH * HEIGHT],
}

struct Client {
    id: usize,
    stream: TcpStream,
}

fn handle_client(client_id: usize, clients: Arc<Mutex<HashMap<usize, Client>>>) {
    let mut buffer = [0; WIDTH * HEIGHT];

    loop {
        let mut should_break = false;
        {

            let mut stream = {
                let clients = clients.lock().unwrap();
                clients[&client_id].stream.try_clone().expect("Failed to clone stream")
            };


                match stream.read(&mut buffer) {
                    Ok(0) => {
                        // The client has closed the connection
                        println!(
                            "Client disconnected: {}",
                            stream.peer_addr().unwrap()
                        );
                        should_break = true;
                    }
                    Ok(_) => {
                        println!("Got something from client {}", client_id);
                        let texture_data = TextureData { data: buffer };

                        // Send updated texture data to all clients
                        let mut clients = clients.lock().unwrap();
                        for client in clients.values_mut() {
                            let _ = client.stream.write(&texture_data.data);
                        }
                    }
                    Err(e) => println!("Failed to read from client: {}", e),
                }

        }

        if should_break {
            let mut locked_clients = clients.lock().unwrap();
            locked_clients.remove(&client_id);
            break;
        }

        thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let mut next_client_id = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let client_id = next_client_id;
                next_client_id += 1;
                let mut locked_clients = clients.lock().unwrap();
                locked_clients.insert(
                    client_id,
                    Client {
                        id: client_id,
                        stream: stream.try_clone().unwrap(),
                    },
                );
                println!("Clients len is now {}", locked_clients.len());
                drop(locked_clients);
                let clients_ref_clone = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_client(client_id, clients_ref_clone);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}