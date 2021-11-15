use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::thread;
use crate::connections::constants::IP;

pub struct MessageReceiver{
}

impl MessageReceiver {
    pub fn start_receiver() { //Habria que pasarle la funcion que queremos que ejecute
    let listener = TcpListener::bind(IP).unwrap();

    for stream in listener.incoming() {
    println ! ("Cliente conectado");
    let mut reader = BufReader::new(stream.unwrap());
    thread::spawn( move | | {

        //Aca deberiamos pasarle la funcion por parametro.
        loop {
    let mut buffer = String::new();
    reader.read_line( & mut buffer);
    if buffer.len() > 0 {
    println ! ("Hello {}", buffer);
    } else {
    println ! ("Goodbye!");
    break;
    }
    }
    });
    }
}
