use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::TextureData;
use crate::history::ChatHistory;
use std::time::{Duration, SystemTime, UNIX_EPOCH};



pub const PACKET_SIZE: usize = 40055;
const MAX_HISTORY: usize = 56;

// #[derive(Clone, Debug)]
// pub struct Connection {
//     pub stream: Arc<Mutex<TcpStream>>
// }

//     pub fn new(address: &String) -> Connection {
//         let stream = TcpStream::connect(address).unwrap();
//         stream.set_nonblocking(true).unwrap();

//         Connection {
//             stream: Arc::new(Mutex::new(stream))
//         }
//     }

    pub fn receive(history: &Arc<Mutex<ChatHistory>>, stream: &Arc<Mutex<TcpStream>>, should_close: &Arc<AtomicBool>) {
        stream.lock().unwrap().set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        while !should_close.load(Ordering::Relaxed) {
            let mut buffer = [0; PACKET_SIZE];
            let mut stream = stream.lock().unwrap();
            match stream.read_exact(&mut buffer) {
                Ok(_) => {
                    let received_texture_data: TextureData = bincode::deserialize(&buffer).unwrap();
                    history.lock().unwrap().history.push(received_texture_data);
                    if history.lock().unwrap().history.len() > MAX_HISTORY {
                        history.lock().unwrap().history.remove(0);
                    }
                    history.lock().unwrap().history.sort_by_key(|item| item.timestamp);
                    history.lock().unwrap().dirty = true;
                    println!("Received drawing from server");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No incoming message
                }
                Err(e) => {
                    //println!("Failed to read from server: {}", e);
                }
            }
        }
    }

    pub fn send(texture_data: &TextureData, stream: &Arc<Mutex<TcpStream>>) {
        let mut stream = stream.lock().unwrap();
        let serialized_data = bincode::serialize(&texture_data).unwrap();
        stream.write_all(&serialized_data).unwrap();
    }
    pub fn confirm_history(myname: &String, stream: &mut TcpStream) {
        let bytes = myname.as_bytes();
        let mut fixed_size_text = [0u8; 24];
        fixed_size_text[..bytes.len()].copy_from_slice(bytes);
        let now = SystemTime::now();

        let texture_data = TextureData {
            name: fixed_size_text,
            data: vec![0; 200*200],
            request_history: false,
            request_history_length: false,
            history_length: 0,
            confirm_history: true,
            timestamp: now.duration_since(UNIX_EPOCH).unwrap().as_millis()
        };
        let serialized_data = bincode::serialize(&texture_data).unwrap();
        stream.write_all(&serialized_data).unwrap();
    }
    pub fn request_history(myname: &String, stream: &mut TcpStream) {
        let bytes = myname.as_bytes();
        let mut fixed_size_text = [0u8; 24];
        fixed_size_text[..bytes.len()].copy_from_slice(bytes);
        let now = SystemTime::now();

        let texture_data = TextureData {
            name: fixed_size_text,
            data: vec![0; 200*200],
            request_history: true,
            request_history_length: false,
            history_length: 0,
            confirm_history: false,
            timestamp: now.duration_since(UNIX_EPOCH).unwrap().as_millis()
        };
        let serialized_data = bincode::serialize(&texture_data).unwrap();
        stream.write_all(&serialized_data).unwrap();
    }
    pub fn request_history_length(myname: &String, stream: &mut TcpStream) {
        let bytes = myname.as_bytes();
        let mut fixed_size_text = [0u8; 24];
        fixed_size_text[..bytes.len()].copy_from_slice(bytes);
        let now = SystemTime::now();

        let texture_data = TextureData {
            name: fixed_size_text,
            data: vec![0; 200*200],
            request_history: false,
            request_history_length: true,
            history_length: 0,
            confirm_history: false,
            timestamp: now.duration_since(UNIX_EPOCH).unwrap().as_millis()
        };
        let serialized_data = bincode::serialize(&texture_data).unwrap();
        stream.write_all(&serialized_data).unwrap();
    }
