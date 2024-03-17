use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use serde::{Serialize, Deserialize};
use bincode::serialized_size;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

const PACKET_SIZE: usize = 40055;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TextureData {
    name: [u8; 24],
    data: Vec<u8>,
    request_history: bool,
    request_history_length: bool,
    history_length: i32,
    confirm_history: bool,
    timestamp: u128
}

struct Client {
    stream: TcpStream,
    has_history: bool
}

struct History {
    history: Vec<TextureData>
}

impl History {
    pub fn new() -> History {
        History {
            history: Vec::new()
        }
    }
}

fn handle_client(client_id: usize, clients: Arc<Mutex<HashMap<usize, Client>>>, history: Arc<Mutex<History>>) {
    let mut buffer = [0; PACKET_SIZE];

    loop {
        let mut should_break = false;
        {

            let mut stream = {
                let clients = clients.lock().unwrap();
                clients[&client_id].stream.try_clone().expect("Failed to clone stream")
            };

                match stream.read_exact(&mut buffer) {

                    Ok(_) => {
                        println!("Got something from client {}", client_id);
                        let texture_data: TextureData = bincode::deserialize(&buffer).unwrap();

                        if texture_data.request_history_length {
                            let history_locked = history.lock().unwrap();
                            let response = TextureData {
                                name: [0u8; 24],
                                data: vec![0u8; 200*200],
                                request_history: false,
                                request_history_length: false,
                                history_length: serialized_size(&((*history_locked).history)).unwrap() as i32,
                                confirm_history: false,
                                timestamp: 0
                            };
                            let serialized_data = bincode::serialize(&response).unwrap();
                            stream.write_all(&serialized_data).unwrap();
                            match stream.read_exact(&mut buffer) {
                                Ok(_) => {
                                    let history_req: TextureData = bincode::deserialize(&buffer).unwrap();
                                    if history_req.request_history {
                                        let history_data = bincode::serialize(&((*history_locked).history)).unwrap();
                                        stream.write_all(&history_data).unwrap();
                                        match stream.read_exact(&mut buffer) {
                                            Ok(_) => {
                                                let confirm: TextureData = bincode::deserialize(&buffer).unwrap();
                                                if confirm.confirm_history {
                                                    let mut clients = clients.lock().unwrap();
                                                    clients.get_mut(&client_id).unwrap().has_history = true;
                                                }
                                            },
                                            Err(e) => {
            
                                            }
                                        }
                                    }
                                },
                                Err(e) => {

                                }
                            }
                        } else {
                            // Add the message to history
                            let mut history_locked = history.lock().unwrap();
                            (*history_locked).history.push(texture_data);
                            (*history_locked).history.sort_by_key(|item| item.timestamp);
                            println!("History len is now {}", (*history_locked).history.len());
                            // Serialize and save (overwrite) to file
                            let file = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open("history")
                                .unwrap();
                            let writer = BufWriter::new(file);
                            bincode::serialize_into(writer, &(*history_locked).history).unwrap();

                            // Send updated texture data to all clients
                            let mut clients = clients.lock().unwrap();
                            for client in clients.values_mut() {
                                if(client.has_history) {
                                    let _ = client.stream.write_all(&buffer);
                                }
                            }
                        }
                        
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::UnexpectedEof {
                            println!(
                                "Client disconnected: {}",
                                stream.peer_addr().unwrap()
                            );
                            should_break = true;
                        } else {
                            println!("Failed to receive from client: {}", e);
                        }
                    }
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
    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let history = Arc::new(Mutex::new(History::new()));

    let save_path = "history";

    if Path::new(save_path).exists() {
        let file = File::open(save_path).unwrap();
        let reader = BufReader::new(file);
        history.lock().unwrap().history = bincode::deserialize_from(reader).unwrap();
        println!("Loaded data.");
    } else {
        println!("File does not exist, initializing new data.");
    }

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
                        stream: stream.try_clone().unwrap(),
                        has_history: false
                    },
                );
                println!("Clients len is now {}", locked_clients.len());
                drop(locked_clients);
                let clients_ref_clone = Arc::clone(&clients);
                let history_ref_clone = Arc::clone(&history);
                thread::spawn(move || {
                    handle_client(client_id, clients_ref_clone, history_ref_clone);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}