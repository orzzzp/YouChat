
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::str::FromStr;
use std::collections::HashMap;

struct PushManager {
	sender: Sender<PushCommand>
}

enum PushCommand {
	NewConnection {id: usize, stream: TcpStream},
	SendMessage {id: usize, message: String}
}

impl PushManager {
	fn new() -> PushManager {
		let (sender, receiver) = channel();
		thread::spawn(move|| {
			let mut map = HashMap::new();
			loop {
				let command = receiver.recv().unwrap();
				match command {
					PushCommand::NewConnection {id, stream} => {
						let (tx, rx) = channel();
						map.insert(id, tx);
						thread::spawn(move|| {
							let message: String = rx.recv().unwrap();
							let mut stream = BufWriter::new(stream);
							stream.write(message.as_ref());
						});

					}
					PushCommand::SendMessage {id, message} => {
						map.get(&id).unwrap().send(message);
					}
				}
			}
		});
		PushManager {sender: sender}
	}

	fn new_connection(&self, id: usize, stream: TcpStream) {
		let command = PushCommand::NewConnection {id: id, stream: stream};
		self.sender.send(command);
	}

	fn send_message(&self, id: usize, message: String) {
		let command = PushCommand::SendMessage {id: id, message: message};
		self.sender.send(command);
	}
}

impl Clone for PushManager {
	fn clone(&self) -> Self {
		PushManager {sender: self.sender.clone()}
	}
}

fn handle_client(stream: TcpStream, push_manager: PushManager) {
	let stream_reader = BufReader::new(&stream);

	for command in stream_reader.lines() {
		match command {
			Ok(command) => {
				println!("command {}", command);
				let args:Vec<&str>  = command.split(' ').collect();
				if args[0] == "login" {
					let id = usize::from_str(args[1]).unwrap();
					push_manager.new_connection(id, stream.try_clone().unwrap());
				} else if args[0] == "message" {
					let id = usize::from_str(args[1]).unwrap();
					let message = args[2].to_string();
					push_manager.send_message(id, message);
				}

			}
			Err(_) => {

			}
		}

	}
}

fn main() {

	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

	let push_manager = PushManager::new();

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let push_manager = push_manager.clone();
				thread::spawn(move|| {
					handle_client(stream, push_manager);
				});
			}
			Err(_) => {

			}
		}
	}
}
