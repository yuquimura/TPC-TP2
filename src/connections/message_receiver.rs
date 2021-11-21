use crate::connections::constants::IP;
use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::thread;

pub struct MessageReceiver {}

impl MessageReceiver {
    #[allow(dead_code)]
    pub fn start_receiver() {
        //Habria que pasarle la funcion que queremos que ejecute
        let listener = TcpListener::bind(IP).unwrap();

        for stream in listener.incoming() {
            println!("Cliente conectado");
            let mut reader = BufReader::new(stream.unwrap());
            thread::spawn(move || {
                //Aca deberiamos pasarle la funcion por parametro.
                loop {
                    let mut buffer = String::new();
                    reader.read_line(&mut buffer).unwrap(); // TODO: Manage this
                    if !buffer.is_empty() {
                        println!("Hello {}", buffer);
                    } else {
                        println!("Goodbye!");
                        break;
                    }
                }
            });
        }
    }
}
