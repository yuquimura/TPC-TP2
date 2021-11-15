use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::thread;
use crate::connections::constants::IP;

fn start_receiver() {
    let listener = TcpListener::bind(IP).unwrap();

    for stream in listener.incoming() {
        println!("Cliente conectado");
        let mut reader = BufReader::new(stream.unwrap());
        thread::spawn(move || {
            loop {
                let mut buffer = String::new();
                reader.read_line(&mut buffer);
                if buffer.len() > 0 {
                    println!("Hello {}", buffer);
                } else {
                    println!("Goodbye!");
                    break;
                }
            }
        });
    }
}