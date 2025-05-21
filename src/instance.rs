use std::{
    fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
};

use crate::constants::{DATA_DIR, SOCKET_NAME};

pub enum InstanceEvent {
    Open(String),
}

pub struct Instance {
    sender: Sender<InstanceEvent>,
    receiver: Receiver<InstanceEvent>,
    socket: Option<UnixStream>,
    socket_path: PathBuf,
}

impl Instance {
    pub fn new() -> Self {
        let runtime_dir = dirs::runtime_dir()
            .expect("Failed to get runtime dir")
            .join(DATA_DIR);

        fs::create_dir_all(&runtime_dir).expect("Failed to create runtime directory");

        let socket_path = runtime_dir.join(SOCKET_NAME);

        let (sender, receiver) = channel::<InstanceEvent>();
        let socket = UnixStream::connect(&socket_path).ok();

        Self {
            sender,
            receiver,
            socket,
            socket_path,
        }
    }

    pub fn running(&self) -> bool {
        self.socket.is_some()
    }

    pub fn send(&self, data: String) {
        if let Some(mut stream) = self.socket.as_ref() {
            stream
                .write_all(data.as_bytes())
                .expect("Failed to write to stream");
        }
    }

    pub fn start(&self) {
        let _ = std::fs::remove_file(&self.socket_path);
        let listener = UnixListener::bind(&self.socket_path).expect("Failed to create socket");

        let sender = self.sender.clone();
        thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut buffer = String::new();
                if stream.read_to_string(&mut buffer).is_ok() {
                    sender.send(InstanceEvent::Open(buffer)).ok();
                }
            }
        });
    }

    pub fn stop(&self) {
        let _ = fs::remove_file(&self.socket_path);
    }

    pub fn events<F: FnMut(InstanceEvent)>(&self, handler: F) {
        self.receiver.try_iter().for_each(handler);
    }
}
