use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::TextureData;

const WIDTH: usize = 200;
const HEIGHT: usize = 200;

#[derive(Clone, Debug)]
pub struct Connection {
    pub stream: Arc<Mutex<TcpStream>>,
    pub history: Vec<TextureData>
}

impl Connection {
    pub fn new() -> Connection {
        let stream = TcpStream::connect("127.0.0.1:7878").unwrap();
        stream.set_nonblocking(true).unwrap();

        Connection {
            stream: Arc::new(Mutex::new(stream)),
            history: Vec::new()
        }
    }

    pub fn receive(&mut self) {
        let mut buffer = [0; WIDTH * HEIGHT];
        let mut stream = self.stream.lock().unwrap();
        match stream.read(&mut buffer) {
            Ok(_) => {
                let received_texture_data = TextureData { data: buffer };
                self.history.push(received_texture_data);
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
        stream.write_all(&texture_data.data).unwrap();
    }
}