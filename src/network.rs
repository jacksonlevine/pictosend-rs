use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::TextureData;
use crate::history::ChatHistory;
use std::time::{SystemTime, UNIX_EPOCH};

pub const PACKET_SIZE: usize = 40055;

#[derive(Clone, Debug)]
pub struct Connection {
    pub stream: Arc<Mutex<TcpStream>>
}

impl Connection {
    pub fn new() -> Connection {
        let stream = TcpStream::connect("127.0.0.1:7878").unwrap();
        stream.set_nonblocking(true).unwrap();

        Connection {
            stream: Arc::new(Mutex::new(stream))
        }
    }

    pub fn receive(&mut self, history: &Arc<Mutex<ChatHistory>>) {
        let mut buffer = [0; PACKET_SIZE];
        let mut stream = self.stream.lock().unwrap();
        match stream.read_exact(&mut buffer) {
            Ok(_) => {
                let received_texture_data: TextureData = bincode::deserialize(&buffer).unwrap();
                history.lock().unwrap().history.push(received_texture_data);
                history.lock().unwrap().history.sort_by_key(|item| item.timestamp);
                history.lock().unwrap().dirty = true;
                println!("Received drawing from server");
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No incoming message
            }
            Err(e) => {
                println!("Failed to read from server: {}", e);
            }
        }
    }

    pub fn send(&mut self, texture_data: &TextureData) {
        let mut stream = self.stream.lock().unwrap();
        let serialized_data = bincode::serialize(&texture_data).unwrap();
        stream.write_all(&serialized_data).unwrap();
    }
    pub fn confirm_history(&mut self, myname: &String, stream: &mut TcpStream) {
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
    pub fn request_history(&mut self, myname: &String, stream: &mut TcpStream) {
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
    pub fn request_history_length(&mut self, myname: &String, stream: &mut TcpStream) {
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
}