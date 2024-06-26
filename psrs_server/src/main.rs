use core::num;
use std::char::MAX;
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
use uuid::Uuid;

const PACKET_SIZE: usize = 40055;
const MAX_HISTORY: usize = 56;

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
    has_history: bool,
    errorstrikes: i8
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

#[derive(Serialize, Deserialize, PartialEq)]
enum InfoMsg {
    RequestHistoryLength,
    HistoryLength,
    RequestHistory,
    ConfirmReceivedHistory,
    Nothing
}
#[derive(Serialize, Deserialize)]
struct InfoData {
    msg: InfoMsg,
    number: i32
}

fn handle_client(client_id: Uuid, clients: Arc<Mutex<HashMap<Uuid, Client>>>, history: Arc<Mutex<History>>) {
    let mut info = InfoData {
        msg: InfoMsg::Nothing,
        number: 0
    };
    let infosize = serialized_size(&info).unwrap() as usize;
    println!("is: {infosize}");
    let mut buffer = [0; PACKET_SIZE];
    let mut cliname = String::new();
    loop {
        let mut should_break = false;
        {

            let mut stream = {
                let clients = clients.lock().unwrap();
                clients[&client_id].stream.try_clone().expect("Failed to clone stream")
            };
            
                match stream.read(&mut buffer) {

                    Ok(numbytes) => {

                        
                        if numbytes == infosize {
                            println!("Got an info data");
                            let info_data: InfoData = bincode::deserialize(&buffer[..infosize]).unwrap();
                            match info_data.msg {
                                InfoMsg::RequestHistoryLength => {
                                                    println!("Got history length request");
                                                    let history_locked = history.lock().unwrap();
                                                    let response = InfoData {
                                                        msg: InfoMsg::HistoryLength,
                                                        number: serialized_size(&((*history_locked).history)).unwrap() as i32,
                                                    };
                                                    let serialized_data = bincode::serialize(&response).unwrap();
                                                    stream.write_all(&serialized_data).unwrap();
                                                    println!("Sent history length, now expecting history request");
                                                    match stream.read(&mut buffer) {
                                                        Ok(bb) => {
                                                            println!("Got {bb} bytes response, gonna read {infosize} bytes of it");
                                                            let history_req: InfoData = bincode::deserialize(&buffer[..infosize]).unwrap();
                                                            if history_req.msg == InfoMsg::RequestHistory {
                                                                println!("It's a history request, sending history");
                                                                let history_data = bincode::serialize(&((*history_locked).history)).unwrap();
                                                                stream.write_all(&history_data).unwrap();
                                                                println!("Sent history");
                                                                match stream.read(&mut buffer) {
                                                                    Ok(bb) => {
                                                                        println!("Got {bb} bytes, expecting it to be confirmation");
                                                                        let confirm: InfoData = bincode::deserialize(&buffer[..infosize]).unwrap();
                                                                        if confirm.msg == InfoMsg::ConfirmReceivedHistory {
                                                                            println!("It was confirmation");
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
                                },
                                InfoMsg::RequestHistory => {},
                                InfoMsg::HistoryLength => {},
                                InfoMsg::ConfirmReceivedHistory => {},
                                InfoMsg::Nothing => {},
                            }

                        } else if numbytes == PACKET_SIZE {


                                                    let texture_data: TextureData = bincode::deserialize(&buffer).unwrap();
                                                    let name = String::from_utf8(texture_data.name.to_vec()).unwrap();
                                                    cliname = name.clone();
                            
                                                    println!("Got something from client {}", name);
                            
                                                    if texture_data.request_history_length {
                                                        
                                                    } else {
                                                        // Add the message to history
                                                        let mut history_locked = history.lock().unwrap();
                                                        (*history_locked).history.push(texture_data);
                                                        if (*history_locked).history.len() > MAX_HISTORY {
                                                            (*history_locked).history.remove(0);
                                                        }
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
                                                            if client.has_history {
                                                                let _ = client.stream.write_all(&buffer);
                                                            }
                                                        }
                                                    }


                        }


                        
                        
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::UnexpectedEof {
                            println!(
                                "Client disconnected: {}",
                                cliname
                            );
                            should_break = true;
                        } else {
                            println!("Failed to receive from client: {}", e);
                            let mut clients = clients.lock().unwrap();
                            clients.get_mut(&client_id).unwrap().errorstrikes += 1;

                            if clients.get_mut(&client_id).unwrap().errorstrikes > 4 {
                                should_break = true;
                            }
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

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let mut locked_clients = clients.lock().unwrap();
                let client_id = Uuid::new_v4();
                locked_clients.insert(
                    client_id,
                    Client {
                        stream: stream.try_clone().unwrap(),
                        has_history: false,
                        errorstrikes: 0
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