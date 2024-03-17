use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::TextureData;
use crate::history::ChatHistory;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::winflash::flash_window;

pub const PACKET_SIZE: usize = 40055;
const MAX_HISTORY: usize = 56;

#[derive(Clone, Debug)]
pub struct Connection {
    pub stream: Arc<Mutex<TcpStream>>,
    pub window_handle: winapi::shared::windef::HWND
}

impl Connection {
    pub fn new(address: &String, window_handle: &winapi::shared::windef::HWND) -> Connection {
        let stream = TcpStream::connect(address).unwrap();
        stream.set_nonblocking(true).unwrap();

        Connection {
            stream: Arc::new(Mutex::new(stream)),
            window_handle: window_handle.clone()
        }
    }

    pub fn receive(&mut self, history: &Arc<Mutex<ChatHistory>>) {
        let mut buffer = [0; PACKET_SIZE];
        let mut stream = self.stream.lock().unwrap();
        match stream.read_exact(&mut buffer) {
            Ok(_) => {
                let received_texture_data: TextureData = bincode::deserialize(&buffer).unwrap();
                history.lock().unwrap().history.push(received_texture_data);
                if history.lock().unwrap().history.len() > MAX_HISTORY {
                    history.lock().unwrap().history.remove(0);
                }
                history.lock().unwrap().history.sort_by_key(|item| item.timestamp);
                history.lock().unwrap().dirty = true;
                flash_window(self.window_handle);
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